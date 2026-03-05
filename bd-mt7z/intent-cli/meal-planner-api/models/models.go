package models

import "time"

// Recipe represents a scraped recipe
type Recipe struct {
	ID               string     `json:"id"`
	Title            string     `json:"title"`
	SourceURL        string     `json:"source_url"`
	Ingredients      []string   `json:"ingredients"`
	Instructions     []string   `json:"instructions"`
	PrepTimeMinutes  int        `json:"prep_time_minutes,omitempty"`
	CookTimeMinutes  int        `json:"cook_time_minutes,omitempty"`
	TotalTimeMinutes int        `json:"total_time_minutes,omitempty"`
	Servings         int        `json:"servings"`
	Nutrition        *Nutrition `json:"nutrition,omitempty"`
	Tags             []string   `json:"tags"`
	CreatedAt        time.Time  `json:"created_at"`
}

// Nutrition contains nutritional information
type Nutrition struct {
	Calories  int `json:"calories,omitempty"`
	ProteinG  int `json:"protein_g,omitempty"`
	CarbsG    int `json:"carbs_g,omitempty"`
	FatG      int `json:"fat_g,omitempty"`
	FiberG    int `json:"fiber_g,omitempty"`
	SodiumMg  int `json:"sodium_mg,omitempty"`
}

// MealPlan represents a meal plan
type MealPlan struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	StartDate string    `json:"start_date"`
	EndDate   string    `json:"end_date"`
	Meals     []Meal    `json:"meals"`
	CreatedAt time.Time `json:"created_at"`
}

// Meal represents a scheduled meal
type Meal struct {
	ID       string       `json:"id"`
	RecipeID string       `json:"recipe_id"`
	Date     string       `json:"date"`
	MealType string       `json:"meal_type"`
	Servings int          `json:"servings"`
	Recipe   *RecipeBrief `json:"recipe,omitempty"`
}

// RecipeBrief is a brief recipe reference
type RecipeBrief struct {
	ID    string `json:"id"`
	Title string `json:"title"`
}

// ShoppingList represents an aggregated shopping list
type ShoppingList struct {
	PlanID      string             `json:"plan_id"`
	Items       []ShoppingListItem `json:"items"`
	GeneratedAt time.Time          `json:"generated_at"`
}

// ShoppingListItem represents a single shopping item
type ShoppingListItem struct {
	Ingredient string   `json:"ingredient"`
	Quantity   string   `json:"quantity"`
	Recipes    []string `json:"recipes"`
}

// APIError represents an error response
type APIError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Hint    string `json:"hint,omitempty"`
}

// ErrorResponse wraps an API error
type ErrorResponse struct {
	Error APIError `json:"error"`
}

// RecipeListResponse for listing recipes
type RecipeListResponse struct {
	Recipes []Recipe `json:"recipes"`
	Total   int      `json:"total"`
}

// RecipeExportResponse for exporting recipes
type RecipeExportResponse struct {
	Recipes    []Recipe  `json:"recipes"`
	ExportedAt time.Time `json:"exported_at"`
	Version    string    `json:"version"`
}

// MealPlanExportResponse for exporting a meal plan
type MealPlanExportResponse struct {
	MealPlan MealPlan `json:"meal_plan"`
	Recipes  []Recipe `json:"recipes"`
}

// ScrapeRequest is the request body for scraping a recipe
type ScrapeRequest struct {
	URL string `json:"url"`
}

// CreateMealPlanRequest is the request body for creating a meal plan
type CreateMealPlanRequest struct {
	Name      string `json:"name"`
	StartDate string `json:"start_date"`
	EndDate   string `json:"end_date"`
}

// AddMealRequest is the request body for adding a meal
type AddMealRequest struct {
	RecipeID string `json:"recipe_id"`
	Date     string `json:"date"`
	MealType string `json:"meal_type"`
	Servings int    `json:"servings"`
}
