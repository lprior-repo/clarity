package store

import (
	"crypto/rand"
	"encoding/hex"
	"sync"

	"github.com/meal-planner-api/models"
)

// Store is an in-memory data store
type Store struct {
	mu        sync.RWMutex
	recipes   map[string]*models.Recipe
	mealPlans map[string]*models.MealPlan
}

// New creates a new Store
func New() *Store {
	return &Store{
		recipes:   make(map[string]*models.Recipe),
		mealPlans: make(map[string]*models.MealPlan),
	}
}

// GenerateID generates a random ID with the given prefix
func GenerateID(prefix string) string {
	b := make([]byte, 8)
	rand.Read(b)
	return prefix + hex.EncodeToString(b)
}

// SaveRecipe saves a recipe
func (s *Store) SaveRecipe(r *models.Recipe) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.recipes[r.ID] = r
}

// GetRecipe retrieves a recipe by ID
func (s *Store) GetRecipe(id string) (*models.Recipe, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	r, ok := s.recipes[id]
	return r, ok
}

// ListRecipes returns all recipes
func (s *Store) ListRecipes() []*models.Recipe {
	s.mu.RLock()
	defer s.mu.RUnlock()
	recipes := make([]*models.Recipe, 0, len(s.recipes))
	for _, r := range s.recipes {
		recipes = append(recipes, r)
	}
	return recipes
}

// DeleteRecipe removes a recipe
func (s *Store) DeleteRecipe(id string) bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, ok := s.recipes[id]; !ok {
		return false
	}
	delete(s.recipes, id)
	return true
}

// SaveMealPlan saves a meal plan
func (s *Store) SaveMealPlan(mp *models.MealPlan) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.mealPlans[mp.ID] = mp
}

// GetMealPlan retrieves a meal plan by ID
func (s *Store) GetMealPlan(id string) (*models.MealPlan, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	mp, ok := s.mealPlans[id]
	return mp, ok
}

// AddMealToPlan adds a meal to a meal plan
func (s *Store) AddMealToPlan(planID string, meal models.Meal) (*models.Meal, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	mp, ok := s.mealPlans[planID]
	if !ok {
		return nil, false
	}
	mp.Meals = append(mp.Meals, meal)
	return &meal, true
}
