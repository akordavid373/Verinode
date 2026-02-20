const express = require('express');
const router = express.Router();
const searchController = require('../controllers/searchController');

// Public routes
router.get('/', searchController.search);
router.get('/autocomplete', searchController.autoComplete);
router.get('/popular', searchController.getPopularSearches);

// Protected routes (require authentication)
router.get('/history', searchController.getSearchHistory);
router.delete('/history/:id', searchController.deleteSearchHistory);
router.delete('/history', searchController.clearSearchHistory);
router.post('/saved', searchController.saveSearchQuery);
router.get('/saved', searchController.getSavedQueries);
router.delete('/saved/:id', searchController.deleteSavedQuery);
router.get('/recent', searchController.getRecentSearches);

// Admin routes (require admin privileges)
router.get('/analytics', searchController.getSearchAnalytics);
router.get('/indexes/stats', searchController.getIndexStats);
router.post('/indexes/rebuild', searchController.rebuildIndexes);
router.get('/indexes/health', searchController.checkIndexHealth);
router.get('/indexes/suggestions', searchController.getIndexSuggestions);

module.exports = router;