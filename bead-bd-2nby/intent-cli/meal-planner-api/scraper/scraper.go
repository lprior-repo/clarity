package scraper

import (
	"encoding/json"
	"errors"
	"net/url"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/gocolly/colly/v2"
	"github.com/meal-planner-api/models"
	"github.com/meal-planner-api/store"
)

var (
	ErrInvalidURL      = errors.New("invalid URL")
	ErrRecipeNotFound  = errors.New("could not extract recipe data from this URL")
	ErrUnsupportedSite = errors.New("unsupported website")
)

// Scraper handles recipe scraping
type Scraper struct{}

// New creates a new Scraper
func New() *Scraper {
	return &Scraper{}
}

// ScrapeRecipe scrapes a recipe from the given URL
func (s *Scraper) ScrapeRecipe(rawURL string) (*models.Recipe, error) {
	// Validate URL
	parsedURL, err := url.Parse(rawURL)
	if err != nil || parsedURL.Scheme == "" || parsedURL.Host == "" {
		return nil, ErrInvalidURL
	}

	// Check if the scheme is http or https
	if parsedURL.Scheme != "http" && parsedURL.Scheme != "https" {
		return nil, ErrInvalidURL
	}

	// Try to scrape using JSON-LD first
	recipe, err := s.scrapeWithJSONLD(rawURL)
	if err == nil && recipe != nil {
		return recipe, nil
	}

	// Try HTML scraping as fallback
	recipe, err = s.scrapeWithHTML(rawURL)
	if err != nil {
		return nil, err
	}

	return recipe, nil
}

// scrapeWithJSONLD tries to extract recipe data from JSON-LD
func (s *Scraper) scrapeWithJSONLD(rawURL string) (*models.Recipe, error) {
	var recipe *models.Recipe

	// Get both www and non-www domain variants
	domain := extractDomain(rawURL)
	wwwDomain := "www." + domain

	c := colly.NewCollector(
		colly.AllowedDomains(domain, wwwDomain),
		colly.UserAgent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
	)

	c.SetRequestTimeout(15 * time.Second)

	c.OnHTML("script[type='application/ld+json']", func(e *colly.HTMLElement) {
		if recipe != nil {
			return // Already found a recipe
		}

		var data interface{}
		if err := json.Unmarshal([]byte(e.Text), &data); err != nil {
			return
		}

		// Handle both single objects and arrays
		switch v := data.(type) {
		case []interface{}:
			for _, item := range v {
				if r := extractRecipeFromLDJSON(item); r != nil {
					recipe = r
					return
				}
			}
		case map[string]interface{}:
			// Check for @graph structure (used by AllRecipes and others)
			if graph, ok := v["@graph"].([]interface{}); ok {
				for _, item := range graph {
					if r := extractRecipeFromLDJSON(item); r != nil {
						recipe = r
						return
					}
				}
			}
			if r := extractRecipeFromLDJSON(v); r != nil {
				recipe = r
			}
		}
	})

	if err := c.Visit(rawURL); err != nil {
		return nil, ErrRecipeNotFound
	}

	if recipe == nil {
		return nil, ErrRecipeNotFound
	}

	recipe.ID = store.GenerateID("rcp_")
	recipe.SourceURL = rawURL
	recipe.CreatedAt = time.Now().UTC()

	// Ensure required fields
	if len(recipe.Tags) == 0 {
		recipe.Tags = []string{}
	}
	if len(recipe.Ingredients) == 0 {
		return nil, ErrRecipeNotFound
	}
	if len(recipe.Instructions) == 0 {
		return nil, ErrRecipeNotFound
	}

	return recipe, nil
}

func extractRecipeFromLDJSON(data interface{}) *models.Recipe {
	obj, ok := data.(map[string]interface{})
	if !ok {
		return nil
	}

	// Check @type
	typeVal, _ := obj["@type"].(string)
	if typeVal != "Recipe" {
		// Check if it's an array
		if types, ok := obj["@type"].([]interface{}); ok {
			isRecipe := false
			for _, t := range types {
				if s, ok := t.(string); ok && s == "Recipe" {
					isRecipe = true
					break
				}
			}
			if !isRecipe {
				return nil
			}
		} else {
			return nil
		}
	}

	recipe := &models.Recipe{
		Servings: 4, // Default
	}

	// Extract title
	if name, ok := obj["name"].(string); ok {
		recipe.Title = name
	}

	// Extract ingredients
	if ingredients, ok := obj["recipeIngredient"].([]interface{}); ok {
		for _, ing := range ingredients {
			if s, ok := ing.(string); ok {
				recipe.Ingredients = append(recipe.Ingredients, s)
			}
		}
	}

	// Extract instructions
	if instructions, ok := obj["recipeInstructions"].([]interface{}); ok {
		for _, inst := range instructions {
			switch v := inst.(type) {
			case string:
				recipe.Instructions = append(recipe.Instructions, v)
			case map[string]interface{}:
				if text, ok := v["text"].(string); ok {
					recipe.Instructions = append(recipe.Instructions, text)
				}
			}
		}
	}

	// Extract times
	recipe.PrepTimeMinutes = parseDuration(obj["prepTime"])
	recipe.CookTimeMinutes = parseDuration(obj["cookTime"])
	recipe.TotalTimeMinutes = parseDuration(obj["totalTime"])

	if recipe.TotalTimeMinutes == 0 && recipe.PrepTimeMinutes+recipe.CookTimeMinutes > 0 {
		recipe.TotalTimeMinutes = recipe.PrepTimeMinutes + recipe.CookTimeMinutes
	}

	// Extract servings
	if yield, ok := obj["recipeYield"].(string); ok {
		if n := extractNumber(yield); n > 0 {
			recipe.Servings = n
		}
	} else if yield, ok := obj["recipeYield"].(float64); ok {
		recipe.Servings = int(yield)
	}

	// Extract tags/keywords
	if keywords, ok := obj["keywords"].(string); ok {
		for _, kw := range strings.Split(keywords, ",") {
			kw = strings.TrimSpace(strings.ToLower(kw))
			if kw != "" {
				recipe.Tags = append(recipe.Tags, kw)
			}
		}
	}

	// Extract nutrition
	if nutrition, ok := obj["nutrition"].(map[string]interface{}); ok {
		recipe.Nutrition = extractNutrition(nutrition)
	}

	return recipe
}

func extractNutrition(data map[string]interface{}) *models.Nutrition {
	n := &models.Nutrition{}

	n.Calories = extractNutrientValue(data["calories"])
	n.ProteinG = extractNutrientValue(data["proteinContent"])
	n.CarbsG = extractNutrientValue(data["carbohydrateContent"])
	n.FatG = extractNutrientValue(data["fatContent"])
	n.FiberG = extractNutrientValue(data["fiberContent"])
	n.SodiumMg = extractNutrientValue(data["sodiumContent"])

	// Only return if we have at least one value
	if n.Calories > 0 || n.ProteinG > 0 || n.CarbsG > 0 || n.FatG > 0 {
		return n
	}
	return nil
}

func extractNutrientValue(v interface{}) int {
	switch val := v.(type) {
	case float64:
		return int(val)
	case string:
		return extractNumber(val)
	}
	return 0
}

func parseDuration(v interface{}) int {
	s, ok := v.(string)
	if !ok {
		return 0
	}

	// Parse ISO 8601 duration (PT30M, PT1H30M, etc.)
	re := regexp.MustCompile(`PT(?:(\d+)H)?(?:(\d+)M)?`)
	matches := re.FindStringSubmatch(s)
	if matches == nil {
		return 0
	}

	minutes := 0
	if matches[1] != "" {
		h, _ := strconv.Atoi(matches[1])
		minutes += h * 60
	}
	if matches[2] != "" {
		m, _ := strconv.Atoi(matches[2])
		minutes += m
	}
	return minutes
}

func extractNumber(s string) int {
	re := regexp.MustCompile(`(\d+)`)
	match := re.FindString(s)
	if match == "" {
		return 0
	}
	n, _ := strconv.Atoi(match)
	return n
}

func extractDomain(rawURL string) string {
	u, err := url.Parse(rawURL)
	if err != nil {
		return ""
	}
	// Remove www. prefix
	host := u.Host
	if strings.HasPrefix(host, "www.") {
		host = host[4:]
	}
	return host
}

// scrapeWithHTML tries to extract recipe data from HTML elements
func (s *Scraper) scrapeWithHTML(rawURL string) (*models.Recipe, error) {
	recipe := &models.Recipe{
		ID:        store.GenerateID("rcp_"),
		SourceURL: rawURL,
		CreatedAt: time.Now().UTC(),
		Servings:  4,
		Tags:      []string{},
	}

	c := colly.NewCollector(
		colly.UserAgent("Mozilla/5.0 (compatible; MealPlannerBot/1.0)"),
	)

	c.SetRequestTimeout(10 * time.Second)

	// Try common recipe title selectors
	c.OnHTML("h1.recipe-title, h1.headline, h1[itemprop='name'], .recipe-header h1", func(e *colly.HTMLElement) {
		if recipe.Title == "" {
			recipe.Title = strings.TrimSpace(e.Text)
		}
	})

	// Try common ingredient selectors
	c.OnHTML("li[itemprop='recipeIngredient'], .ingredient, .ingredients-item", func(e *colly.HTMLElement) {
		text := strings.TrimSpace(e.Text)
		if text != "" {
			recipe.Ingredients = append(recipe.Ingredients, text)
		}
	})

	// Try common instruction selectors
	c.OnHTML("li[itemprop='recipeInstructions'], .instruction, .instructions-section-item", func(e *colly.HTMLElement) {
		text := strings.TrimSpace(e.Text)
		if text != "" {
			recipe.Instructions = append(recipe.Instructions, text)
		}
	})

	if err := c.Visit(rawURL); err != nil {
		return nil, ErrRecipeNotFound
	}

	// Validate we got required fields
	if recipe.Title == "" || len(recipe.Ingredients) == 0 || len(recipe.Instructions) == 0 {
		return nil, ErrRecipeNotFound
	}

	return recipe, nil
}
