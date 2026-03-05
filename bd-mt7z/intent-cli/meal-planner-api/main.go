package main

import (
	"log"
	"net/http"

	"github.com/meal-planner-api/handlers"
	"github.com/meal-planner-api/store"
)

func main() {
	// Initialize in-memory store
	s := store.New()

	// Create handlers with the store
	h := handlers.New(s)

	// Setup routes
	mux := http.NewServeMux()

	// Recipe routes
	mux.HandleFunc("POST /recipes", h.CreateRecipe)
	mux.HandleFunc("POST /recipes/scrape", h.ScrapeRecipe)
	mux.HandleFunc("GET /recipes", h.ListRecipes)
	mux.HandleFunc("GET /recipes/{id}", h.GetRecipe)
	mux.HandleFunc("DELETE /recipes/{id}", h.DeleteRecipe)

	// Meal plan routes
	mux.HandleFunc("POST /meal-plans", h.CreateMealPlan)
	mux.HandleFunc("GET /meal-plans/{id}", h.GetMealPlan)
	mux.HandleFunc("POST /meal-plans/{id}/meals", h.AddMealToPlan)
	mux.HandleFunc("GET /meal-plans/{id}/shopping-list", h.GetShoppingList)

	// Export routes
	mux.HandleFunc("GET /export/recipes", h.ExportRecipes)
	mux.HandleFunc("GET /export/meal-plans/{id}", h.ExportMealPlan)

	// Wrap with middleware
	handler := corsMiddleware(jsonMiddleware(mux))

	log.Println("Meal Planner API starting on http://localhost:8080")
	if err := http.ListenAndServe(":8080", handler); err != nil {
		log.Fatal(err)
	}
}

func jsonMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		next.ServeHTTP(w, r)
	})
}

func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")

		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}

		next.ServeHTTP(w, r)
	})
}
