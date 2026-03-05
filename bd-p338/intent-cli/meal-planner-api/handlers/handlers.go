package handlers

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/meal-planner-api/models"
	"github.com/meal-planner-api/scraper"
	"github.com/meal-planner-api/store"
)

// Handlers contains all HTTP handlers
type Handlers struct {
	store   *store.Store
	scraper *scraper.Scraper
}

// New creates a new Handlers instance
func New(s *store.Store) *Handlers {
	return &Handlers{
		store:   s,
		scraper: scraper.New(),
	}
}

// CreateRecipe handles POST /recipes - direct recipe creation for testing
func (h *Handlers) CreateRecipe(w http.ResponseWriter, r *http.Request) {
	var recipe models.Recipe
	if err := json.NewDecoder(r.Body).Decode(&recipe); err != nil {
		writeError(w, http.StatusBadRequest, "INVALID_REQUEST", "Invalid JSON body", "")
		return
	}

	// Generate ID if not provided
	if recipe.ID == "" {
		recipe.ID = store.GenerateID("rcp_")
	}

	// Set created_at if not provided
	if recipe.CreatedAt.IsZero() {
		recipe.CreatedAt = time.Now().UTC()
	}

	// Ensure arrays are initialized
	if recipe.Ingredients == nil {
		recipe.Ingredients = []string{}
	}
	if recipe.Instructions == nil {
		recipe.Instructions = []string{}
	}
	if recipe.Tags == nil {
		recipe.Tags = []string{}
	}

	// Validate required fields
	if recipe.Title == "" {
		writeError(w, http.StatusBadRequest, "MISSING_TITLE", "Recipe title is required", "")
		return
	}
	if len(recipe.Ingredients) == 0 {
		writeError(w, http.StatusBadRequest, "MISSING_INGREDIENTS", "Recipe must have at least one ingredient", "")
		return
	}
	if len(recipe.Instructions) == 0 {
		writeError(w, http.StatusBadRequest, "MISSING_INSTRUCTIONS", "Recipe must have at least one instruction", "")
		return
	}
	if recipe.Servings < 1 {
		recipe.Servings = 4
	}

	h.store.SaveRecipe(&recipe)

	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(recipe)
}

// ScrapeRecipe handles POST /recipes/scrape
func (h *Handlers) ScrapeRecipe(w http.ResponseWriter, r *http.Request) {
	var req models.ScrapeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "INVALID_REQUEST", "Invalid JSON body", "")
		return
	}

	recipe, err := h.scraper.ScrapeRecipe(req.URL)
	if err != nil {
		switch {
		case errors.Is(err, scraper.ErrInvalidURL):
			writeError(w, http.StatusBadRequest, "INVALID_URL", "The provided URL is not valid", "")
		case errors.Is(err, scraper.ErrRecipeNotFound):
			writeError(w, http.StatusUnprocessableEntity, "RECIPE_NOT_FOUND",
				"Could not extract recipe data from this URL",
				"Try a URL from AllRecipes, Food Network, or BBC Good Food")
		case errors.Is(err, scraper.ErrUnsupportedSite):
			writeError(w, http.StatusUnprocessableEntity, "UNSUPPORTED_SITE",
				"This website is not supported for recipe scraping",
				"Try a URL from AllRecipes, Food Network, or BBC Good Food")
		default:
			writeError(w, http.StatusUnprocessableEntity, "RECIPE_NOT_FOUND",
				"Could not extract recipe data from this URL",
				"Try a URL from AllRecipes, Food Network, or BBC Good Food")
		}
		return
	}

	h.store.SaveRecipe(recipe)

	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(recipe)
}

// ListRecipes handles GET /recipes
func (h *Handlers) ListRecipes(w http.ResponseWriter, r *http.Request) {
	recipes := h.store.ListRecipes()

	// Convert to value slice for JSON
	recipeValues := make([]models.Recipe, len(recipes))
	for i, r := range recipes {
		recipeValues[i] = *r
	}

	resp := models.RecipeListResponse{
		Recipes: recipeValues,
		Total:   len(recipeValues),
	}

	json.NewEncoder(w).Encode(resp)
}

// GetRecipe handles GET /recipes/{id}
func (h *Handlers) GetRecipe(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	recipe, ok := h.store.GetRecipe(id)
	if !ok {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Recipe not found", "")
		return
	}

	json.NewEncoder(w).Encode(recipe)
}

// DeleteRecipe handles DELETE /recipes/{id}
func (h *Handlers) DeleteRecipe(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	if !h.store.DeleteRecipe(id) {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Recipe not found", "")
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

// CreateMealPlan handles POST /meal-plans
func (h *Handlers) CreateMealPlan(w http.ResponseWriter, r *http.Request) {
	var req models.CreateMealPlanRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "INVALID_REQUEST", "Invalid JSON body", "")
		return
	}

	// Validate dates
	startDate, err1 := time.Parse("2006-01-02", req.StartDate)
	endDate, err2 := time.Parse("2006-01-02", req.EndDate)

	if err1 != nil || err2 != nil {
		writeError(w, http.StatusBadRequest, "INVALID_DATE_FORMAT",
			"Dates must be in YYYY-MM-DD format", "")
		return
	}

	if endDate.Before(startDate) {
		writeError(w, http.StatusBadRequest, "INVALID_DATE_RANGE",
			"End date must be after start date", "")
		return
	}

	plan := &models.MealPlan{
		ID:        store.GenerateID("plan_"),
		Name:      req.Name,
		StartDate: req.StartDate,
		EndDate:   req.EndDate,
		Meals:     []models.Meal{},
		CreatedAt: time.Now().UTC(),
	}

	h.store.SaveMealPlan(plan)

	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(plan)
}

// GetMealPlan handles GET /meal-plans/{id}
func (h *Handlers) GetMealPlan(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	plan, ok := h.store.GetMealPlan(id)
	if !ok {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Meal plan not found", "")
		return
	}

	json.NewEncoder(w).Encode(plan)
}

// AddMealToPlan handles POST /meal-plans/{id}/meals
func (h *Handlers) AddMealToPlan(w http.ResponseWriter, r *http.Request) {
	planID := r.PathValue("id")

	var req models.AddMealRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "INVALID_REQUEST", "Invalid JSON body", "")
		return
	}

	// Validate meal type
	validMealTypes := map[string]bool{
		"breakfast": true,
		"lunch":     true,
		"dinner":    true,
		"snack":     true,
	}
	if !validMealTypes[req.MealType] {
		writeError(w, http.StatusBadRequest, "INVALID_MEAL_TYPE",
			"Meal type must be breakfast, lunch, dinner, or snack", "")
		return
	}

	// Validate servings
	if req.Servings < 1 {
		req.Servings = 1
	}

	// Check recipe exists
	recipe, ok := h.store.GetRecipe(req.RecipeID)
	if !ok {
		writeError(w, http.StatusBadRequest, "RECIPE_NOT_FOUND",
			"The specified recipe does not exist", "")
		return
	}

	meal := models.Meal{
		ID:       store.GenerateID("meal_"),
		RecipeID: req.RecipeID,
		Date:     req.Date,
		MealType: req.MealType,
		Servings: req.Servings,
		Recipe: &models.RecipeBrief{
			ID:    recipe.ID,
			Title: recipe.Title,
		},
	}

	_, ok = h.store.AddMealToPlan(planID, meal)
	if !ok {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Meal plan not found", "")
		return
	}

	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(meal)
}

// GetShoppingList handles GET /meal-plans/{id}/shopping-list
func (h *Handlers) GetShoppingList(w http.ResponseWriter, r *http.Request) {
	planID := r.PathValue("id")

	plan, ok := h.store.GetMealPlan(planID)
	if !ok {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Meal plan not found", "")
		return
	}

	// Aggregate ingredients from all meals
	ingredientMap := make(map[string]*models.ShoppingListItem)

	for _, meal := range plan.Meals {
		recipe, ok := h.store.GetRecipe(meal.RecipeID)
		if !ok {
			continue
		}

		for _, ing := range recipe.Ingredients {
			// Normalize ingredient name (simple approach)
			normalized := strings.ToLower(strings.TrimSpace(ing))

			if item, exists := ingredientMap[normalized]; exists {
				// Check if recipe is already in the list
				found := false
				for _, r := range item.Recipes {
					if r == recipe.Title {
						found = true
						break
					}
				}
				if !found {
					item.Recipes = append(item.Recipes, recipe.Title)
				}
			} else {
				ingredientMap[normalized] = &models.ShoppingListItem{
					Ingredient: ing,
					Quantity:   "as needed",
					Recipes:    []string{recipe.Title},
				}
			}
		}
	}

	// Convert map to slice
	items := make([]models.ShoppingListItem, 0, len(ingredientMap))
	for _, item := range ingredientMap {
		items = append(items, *item)
	}

	shoppingList := models.ShoppingList{
		PlanID:      planID,
		Items:       items,
		GeneratedAt: time.Now().UTC(),
	}

	json.NewEncoder(w).Encode(shoppingList)
}

// ExportRecipes handles GET /export/recipes
func (h *Handlers) ExportRecipes(w http.ResponseWriter, r *http.Request) {
	recipes := h.store.ListRecipes()

	// Convert to value slice for JSON
	recipeValues := make([]models.Recipe, len(recipes))
	for i, r := range recipes {
		recipeValues[i] = *r
	}

	resp := models.RecipeExportResponse{
		Recipes:    recipeValues,
		ExportedAt: time.Now().UTC(),
		Version:    "1.0",
	}

	json.NewEncoder(w).Encode(resp)
}

// ExportMealPlan handles GET /export/meal-plans/{id}
func (h *Handlers) ExportMealPlan(w http.ResponseWriter, r *http.Request) {
	planID := r.PathValue("id")

	plan, ok := h.store.GetMealPlan(planID)
	if !ok {
		writeError(w, http.StatusNotFound, "NOT_FOUND", "Meal plan not found", "")
		return
	}

	// Collect all recipes used in the meal plan
	recipeMap := make(map[string]*models.Recipe)
	for _, meal := range plan.Meals {
		if recipe, ok := h.store.GetRecipe(meal.RecipeID); ok {
			recipeMap[recipe.ID] = recipe
		}
	}

	recipes := make([]models.Recipe, 0, len(recipeMap))
	for _, r := range recipeMap {
		recipes = append(recipes, *r)
	}

	resp := models.MealPlanExportResponse{
		MealPlan: *plan,
		Recipes:  recipes,
	}

	json.NewEncoder(w).Encode(resp)
}

// writeError writes a structured error response
func writeError(w http.ResponseWriter, status int, code, message, hint string) {
	w.WriteHeader(status)
	resp := models.ErrorResponse{
		Error: models.APIError{
			Code:    code,
			Message: message,
			Hint:    hint,
		},
	}
	json.NewEncoder(w).Encode(resp)
}
