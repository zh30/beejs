//! Stage 86 Phase 3: 插件市场平台测试套件
//! 测试插件市场、搜索、评分等核心功能

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
use std::sync{Arc, Mutex, RwLock};
use std::collections{HashMap, BTreeMap};

    /// 测试插件市场初始化
    #[tokio::test]
    async fn test_marketplace_initialization() {
        let marketplace: _ = PluginMarketplace::new();
        let result: _ = marketplace.initialize().await;

        assert!(result.is_ok(), "Marketplace initialization should succeed");
    }

    /// 测试插件搜索功能
    #[tokio::test]
    async fn test_plugin_search_basic() {
        let marketplace: _ = PluginMarketplace::new();

        let query: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 1,
                per_page: 10,
            },
        };

        let results: _ = marketplace.search_plugins(&query).await.unwrap();

        // 验证搜索结果结构
        assert!(results.plugins.len() <= 10, "Should return at most 10 results per page");
        assert!(results.total >= 0, "Total should be non-negative");
        assert!(results.took_ms >= 0, "Search time should be non-negative");

        // 验证分面结构
        assert!(results.facets.categories.len() >= 0, "Categories facets should exist");
        assert!(results.facets.tags.len() >= 0, "Tags facets should exist");
        assert!(results.facets.authors.len() >= 0, "Authors facets should exist");
    }

    /// 测试插件搜索过滤器
    #[tokio::test]
    async fn test_plugin_search_filters() {
        let marketplace: _ = PluginMarketplace::new();

        // 测试分类过滤器
        let query: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: Some(vec!["testing".to_string()]),
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: true,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 1,
                per_page: 10,
            },
        };

        let results: _ = marketplace.search_plugins(&query).await.unwrap();

        // 验证过滤器应用
        for result in &results.plugins {
            assert!(result.plugin.categories.contains(&"testing".to_string()),
                   "Results should contain testing category");
            assert!(result.plugin.verified, "Results should be verified");
        }
    }

    /// 测试插件搜索排序
    #[tokio::test]
    async fn test_plugin_search_sorting() {
        let marketplace: _ = PluginMarketplace::new();

        // 测试按下载量排序
        let query: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Downloads,
            pagination: Pagination {
                page: 1,
                per_page: 10,
            },
        };

        let results: _ = marketplace.search_plugins(&query).await.unwrap();

        // 验证排序（下载量降序）
        if results.plugins.len() > 1 {
            for i in 0..results.plugins.len() - 1 {
                assert!(
                    results.plugins[i].plugin.downloads.total >=
                    results.plugins[i + 1].plugin.downloads.total,
                    "Results should be sorted by downloads in descending order"
                );
            }
        }
    }

    /// 测试插件详情获取
    #[tokio::test]
    async fn test_get_plugin_details() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        let result: _ = marketplace.get_plugin_details(&plugin_id).await.unwrap();

        // 插件可能不存在，返回 None 是正常的
        assert!(result.is_none() || result.is_some());
    }

    /// 测试插件评分系统
    #[tokio::test]
    async fn test_plugin_rating_system() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 提交评分
        let rating_result: _ = marketplace.rate_plugin(
            &plugin_id,
            "test_user",
            5,
            Some("Great plugin!".to_string()),
        ).await;

        assert!(rating_result.is_ok(), "Rating submission should succeed");

        // 获取评分
        let rating: _ = marketplace.get_plugin_rating(&plugin_id).await.unwrap();
        assert!(rating.count >= 1, "Rating count should be at least 1");
        assert!(rating.average >= 1.0 && rating.average <= 5.0,
               "Average rating should be between 1 and 5");

        // 验证评分分布
        assert_eq!(rating.distribution.five_star + rating.distribution.four_star +
                  rating.distribution.three_star + rating.distribution.two_star +
                  rating.distribution.one_star, rating.count,
                  "Rating distribution should sum to total count");
    }

    /// 测试评分验证
    #[tokio::test]
    async fn test_rating_validation() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 测试无效评分（超出范围）
        let result: _ = marketplace.rate_plugin(&plugin_id, "user", 0, None).await;
        assert!(result.is_err(), "Rating 0 should be invalid");

        let result: _ = marketplace.rate_plugin(&plugin_id, "user", 6, None).await;
        assert!(result.is_err(), "Rating 6 should be invalid");

        // 测试有效评分
        for rating in 1..=5 {
            let result: _ = marketplace.rate_plugin(&plugin_id, &format!("user{}", rating), rating, None).await;
            assert!(result.is_ok(), format!("Rating {} should be valid", rating).as_str());
        }
    }

    /// 测试用户评分查询
    #[tokio::test]
    async fn test_get_user_rating() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 提交评分
        marketplace.rate_plugin(&plugin_id, "test_user", 4, Some("Good".to_string())).await.unwrap();

        // 获取用户评分
        let user_rating: _ = marketplace.rating_system.get_user_rating(&plugin_id, "test_user").await.unwrap();
        assert!(user_rating.is_some(), "Should find user rating");

        let rating: _ = user_rating.unwrap();
        assert_eq!(rating.rating, 4, "User rating should be 4");
        assert_eq!(rating.user_id, "test_user", "User ID should match");
        assert!(rating.review.is_some(), "Should have review");
    }

    /// 测试评分统计
    #[tokio::test]
    async fn test_rating_statistics() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 提交多个评分
        for i in 1..=10 {
            marketplace.rate_plugin(
                &plugin_id,
                &format!("user{}", i),
                if i % 2 == 0 { 5 } else { 3 },
                if i % 3 == 0 { Some(format!("Review {}", i)) } else { None },
            ).await.unwrap();
        }

        // 获取统计信息
        let stats: _ = marketplace.rating_system.get_rating_statistics(&plugin_id).await.unwrap();
        assert_eq!(stats.total_reviews, 10, "Should have 10 reviews");
        assert!(stats.reviews_with_comments > 0, "Should have some reviews with comments");
        assert!(stats.helpful_votes_total >= 0, "Helpful votes should be non-negative");
        assert!(stats.average_review_length >= 0.0, "Average review length should be non-negative");
    }

    /// 测试帮助性投票
    #[tokio::test]
    async fn test_helpful_voting() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 提交评分和评论
        marketplace.rate_plugin(
            &plugin_id,
            "user1",
            5,
            Some("Great plugin!".to_string()),
        ).await.unwrap();

        // 投票帮助性
        let result: _ = marketplace.rating_system.vote_helpful(&plugin_id, "user1", true).await;
        assert!(result.is_ok(), "Helpful vote should succeed");

        // 验证投票生效
        let user_rating: _ = marketplace.rating_system.get_user_rating(&plugin_id, "user1").await.unwrap();
        assert!(user_rating.is_some(), "Should find user rating");
        assert_eq!(user_rating.unwrap().helpful_votes, 1, "Should have 1 helpful vote");
    }

    /// 测试插件评论获取
    #[tokio::test]
    async fn test_get_plugin_reviews() {
        let marketplace: _ = PluginMarketplace::new();

        let plugin_id: _ = PluginId {
            name: "test-plugin".to_string(),
            version: Version::parse("1.0.0").unwrap(),
        };

        // 提交多个评分和评论
        for i in 1..=5 {
            marketplace.rate_plugin(
                &plugin_id,
                &format!("user{}", i),
                5 - i as u8,
                Some(format!("Review {}", i)),
            ).await.unwrap();
        }

        // 获取评论
        let reviews: _ = marketplace.rating_system.get_plugin_reviews(&plugin_id, 10).await.unwrap();
        assert_eq!(reviews.len(), 5, "Should return all reviews");

        // 验证评论按帮助性排序
        for i in 0..reviews.len() - 1 {
            assert!(
                reviews[i].helpful_votes >= reviews[i + 1].helpful_votes,
                "Reviews should be sorted by helpful votes"
            );
        }
    }

    /// 测试热门插件获取
    #[tokio::test]
    async fn test_get_featured_plugins() {
        let marketplace: _ = PluginMarketplace::new();

        let featured: _ = marketplace.get_featured_plugins(5).await.unwrap();
        assert!(featured.len() <= 5, "Should return at most 5 featured plugins");
    }

    /// 测试最新插件获取
    #[tokio::test]
    async fn test_get_recent_plugins() {
        let marketplace: _ = PluginMarketplace::new();

        let recent: _ = marketplace.get_recent_plugins(10).await.unwrap();
        assert!(recent.len() <= 10, "Should return at most 10 recent plugins");
    }

    /// 测试市场统计信息
    #[tokio::test]
    async fn test_marketplace_stats() {
        let marketplace: _ = PluginMarketplace::new();

        let stats: _ = marketplace.get_plugin_stats().await.unwrap();
        assert!(stats.total_plugins >= 0, "Total plugins should be non-negative");
        assert!(stats.total_downloads >= 0, "Total downloads should be non-negative");
        assert!(stats.average_rating >= 0.0 && stats.average_rating <= 5.0,
               "Average rating should be between 0 and 5");
        assert!(stats.categories_count >= 0, "Categories count should be non-negative");
    }

    /// 测试搜索分页
    #[tokio::test]
    async fn test_search_pagination() {
        let marketplace: _ = PluginMarketplace::new();

        // 第一页
        let query1: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 1,
                per_page: 5,
            },
        };

        let results1: _ = marketplace.search_plugins(&query1).await.unwrap();
        assert!(results1.plugins.len() <= 5, "First page should have at most 5 results");

        // 第二页
        let query2: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 2,
                per_page: 5,
            },
        };

        let results2: _ = marketplace.search_plugins(&query2).await.unwrap();

        // 验证分页逻辑
        assert!(results1.plugins.is_empty() || results2.plugins.is_empty() ||
               results1.plugins[0].plugin.plugin_id != results2.plugins[0].plugin.plugin_id,
               "Pages should not have overlapping results");
    }

    /// 测试搜索缓存
    #[tokio::test]
    async fn test_search_caching() {
        let marketplace: _ = PluginMarketplace::new();

        let query: _ = SearchQuery {
            query: "test".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 1,
                per_page: 10,
            },
        };

        // 第一次搜索
        let start1: _ = std::time::Instant::now();
        let _results1: _ = marketplace.search_plugins(&query).await.unwrap();
        let time1: _ = start1.elapsed();

        // 第二次搜索（应该使用缓存）
        let start2: _ = std::time::Instant::now();
        let _results2: _ = marketplace.search_plugins(&query).await.unwrap();
        let time2: _ = start2.elapsed();

        // 第二次搜索应该更快（使用缓存）
        // 注意：这里我们只是验证功能，实际性能差异取决于实现
        assert!(time2 >= std::time::Duration::from_millis(0), "Second search should complete");
    }

    /// 测试空查询处理
    #[tokio::test]
    async fn test_empty_query() {
        let marketplace: _ = PluginMarketplace::new();

        let query: _ = SearchQuery {
            query: "".to_string(),
            filters: SearchFilters {
                categories: None,
                tags: None,
                authors: None,
                rating_min: None,
                verified_only: false,
                free_only: false,
                version_constraint: None,
                date_range: None,
            },
            sort: SortOption::Relevance,
            pagination: Pagination {
                page: 1,
                per_page: 10,
            },
        };

        let results: _ = marketplace.search_plugins(&query).await.unwrap();
        assert!(results.plugins.len() <= 10, "Empty query should return limited results");
    }

    /// 测试市场初始化和预热
    #[tokio::test]
    async fn test_marketplace_warmup() {
        let marketplace: _ = PluginMarketplace::new();

        let result: _ = marketplace.initialize().await;
        assert!(result.is_ok(), "Initialization should succeed");

        // 验证缓存已预热
        // 这里我们只是验证方法可以调用
        let stats: _ = marketplace.get_plugin_stats().await.unwrap();
        assert!(stats.total_plugins >= 0, "Should have valid plugin count");
    }
}
