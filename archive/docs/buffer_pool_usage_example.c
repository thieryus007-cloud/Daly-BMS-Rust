/**
 * @file buffer_pool_usage_example.c
 * @brief Example usage of WebSocket buffer pool API
 *
 * This file demonstrates how to use the WebSocket buffer pool
 * for high-performance memory allocation with O(1) complexity.
 */

#include "web_server_websocket.h"
#include "esp_log.h"

static const char *TAG = "buffer_pool_example";

/**
 * Example 1: Basic allocation and deallocation
 */
void example_basic_usage(void)
{
    // Allocate a buffer from the pool (O(1) operation)
    char *buffer = ws_buffer_pool_alloc(2048);
    if (buffer == NULL) {
        ESP_LOGE(TAG, "Failed to allocate buffer");
        return;
    }

    // Use the buffer
    snprintf(buffer, 2048, "{\"message\":\"Hello WebSocket\"}");
    ESP_LOGI(TAG, "Buffer content: %s", buffer);

    // Free the buffer back to pool (O(1) operation)
    ws_buffer_pool_free(buffer);
}

/**
 * Example 2: WebSocket message wrapping
 */
void example_wrap_message(const char *payload, size_t payload_len)
{
    // Allocate buffer for wrapped message
    const size_t wrapped_size = payload_len + 64;  // Extra space for wrapper
    char *wrapped = ws_buffer_pool_alloc(wrapped_size);
    if (wrapped == NULL) {
        ESP_LOGW(TAG, "Pool exhausted, falling back to malloc");
        wrapped = malloc(wrapped_size);
        if (wrapped == NULL) {
            ESP_LOGE(TAG, "Malloc failed");
            return;
        }
    }

    // Wrap the payload in JSON
    int written = snprintf(wrapped, wrapped_size,
                          "{\"type\":\"event\",\"data\":%.*s}",
                          (int)payload_len, payload);

    if (written > 0 && (size_t)written < wrapped_size) {
        ESP_LOGI(TAG, "Wrapped message: %s", wrapped);
        // Broadcast wrapped message here...
    }

    // Free the buffer (automatically detects pool vs malloc)
    ws_buffer_pool_free(wrapped);
}

/**
 * Example 3: Getting and displaying statistics
 */
void example_display_statistics(void)
{
    ws_buffer_pool_stats_t stats;
    ws_buffer_pool_get_stats(&stats);

    ESP_LOGI(TAG, "=== Buffer Pool Statistics ===");
    ESP_LOGI(TAG, "Total allocations:    %u", stats.total_allocs);
    ESP_LOGI(TAG, "Pool hits:            %u", stats.pool_hits);
    ESP_LOGI(TAG, "Pool misses:          %u", stats.pool_misses);
    ESP_LOGI(TAG, "Peak usage:           %u/8 buffers", stats.peak_usage);
    ESP_LOGI(TAG, "Current usage:        %u/8 buffers", stats.current_usage);

    if (stats.total_allocs > 0) {
        float hit_rate = (stats.pool_hits * 100.0f) / stats.total_allocs;
        ESP_LOGI(TAG, "Hit rate:             %.1f%%", hit_rate);

        if (hit_rate < 90.0f) {
            ESP_LOGW(TAG, "Low hit rate! Consider increasing pool size or buffer size.");
        }

        if (stats.peak_usage >= 8) {
            ESP_LOGW(TAG, "Pool fully utilized! Consider increasing pool size.");
        }
    }
}

/**
 * Example 4: Periodic statistics monitoring task
 */
void buffer_pool_monitor_task(void *pvParameters)
{
    TickType_t xLastWakeTime = xTaskGetTickCount();
    const TickType_t xFrequency = pdMS_TO_TICKS(30000); // Every 30 seconds

    while (1) {
        vTaskDelayUntil(&xLastWakeTime, xFrequency);

        ws_buffer_pool_stats_t stats;
        ws_buffer_pool_get_stats(&stats);

        // Log statistics
        ESP_LOGI(TAG, "Pool: %u allocs, %u hits (%.1f%%), %u/%u peak usage",
                 stats.total_allocs,
                 stats.pool_hits,
                 stats.total_allocs > 0 ? (stats.pool_hits * 100.0f / stats.total_allocs) : 0.0f,
                 stats.peak_usage,
                 8);

        // Alert on anomalies
        if (stats.total_allocs > 0) {
            float hit_rate = (stats.pool_hits * 100.0f) / stats.total_allocs;
            if (hit_rate < 80.0f) {
                ESP_LOGW(TAG, "Pool hit rate below 80%% - check buffer sizes");
            }
        }

        if (stats.current_usage >= 6) {
            ESP_LOGW(TAG, "Pool usage high (%u/8) - potential exhaustion",
                     stats.current_usage);
        }
    }
}

/**
 * Example 5: Handling large buffers (fallback to malloc)
 */
void example_large_buffer(void)
{
    // Request buffer larger than pool buffer size (4096 bytes)
    size_t large_size = 8192;
    char *large_buffer = ws_buffer_pool_alloc(large_size);

    if (large_buffer == NULL) {
        ESP_LOGE(TAG, "Failed to allocate large buffer");
        return;
    }

    // This will automatically use malloc() and log a debug message
    // about exceeding pool buffer size
    ESP_LOGI(TAG, "Allocated large buffer (%zu bytes) via fallback", large_size);

    // Use the buffer...
    memset(large_buffer, 0, large_size);

    // Free automatically detects malloc vs pool
    ws_buffer_pool_free(large_buffer);
}

/**
 * Example 6: Error handling and edge cases
 */
void example_error_handling(void)
{
    // Allocating zero bytes
    char *buffer1 = ws_buffer_pool_alloc(0);
    if (buffer1 == NULL) {
        ESP_LOGI(TAG, "Zero-size allocation returned NULL (expected)");
    }

    // Freeing NULL pointer (safe)
    ws_buffer_pool_free(NULL);
    ESP_LOGI(TAG, "Freeing NULL is safe (no-op)");

    // Double free detection is not built-in - avoid double frees!
    char *buffer2 = ws_buffer_pool_alloc(1024);
    if (buffer2 != NULL) {
        ws_buffer_pool_free(buffer2);
        // DON'T DO THIS: ws_buffer_pool_free(buffer2);  // DOUBLE FREE!
    }

    // Pool exhaustion scenario
    char *buffers[10];
    int allocated = 0;

    for (int i = 0; i < 10; i++) {
        buffers[i] = ws_buffer_pool_alloc(4096);
        if (buffers[i] != NULL) {
            allocated++;
        }
    }

    ESP_LOGI(TAG, "Allocated %d/10 buffers (pool has 8, rest use malloc)", allocated);

    // Free all
    for (int i = 0; i < allocated; i++) {
        ws_buffer_pool_free(buffers[i]);
    }
}

/**
 * Example 7: Performance comparison
 */
void example_performance_test(void)
{
    const int iterations = 1000;
    int64_t pool_time, malloc_time;

    // Test pool allocation
    int64_t start = esp_timer_get_time();
    for (int i = 0; i < iterations; i++) {
        char *buf = ws_buffer_pool_alloc(2048);
        if (buf != NULL) {
            ws_buffer_pool_free(buf);
        }
    }
    pool_time = esp_timer_get_time() - start;

    // Test malloc/free
    start = esp_timer_get_time();
    for (int i = 0; i < iterations; i++) {
        char *buf = malloc(2048);
        if (buf != NULL) {
            free(buf);
        }
    }
    malloc_time = esp_timer_get_time() - start;

    ESP_LOGI(TAG, "=== Performance Test (%d iterations) ===", iterations);
    ESP_LOGI(TAG, "Pool:   %lld µs (%.2f µs/op)", pool_time, pool_time / (float)iterations);
    ESP_LOGI(TAG, "Malloc: %lld µs (%.2f µs/op)", malloc_time, malloc_time / (float)iterations);
    ESP_LOGI(TAG, "Speedup: %.1fx faster", malloc_time / (float)pool_time);
}

/**
 * Example 8: Integration with WebSocket broadcasting
 */
void example_websocket_integration(void)
{
    // Typical WebSocket message pattern
    const char *telemetry_data = "{\"battery\":{\"voltage\":12.5,\"current\":5.2}}";
    size_t data_len = strlen(telemetry_data);

    // Allocate buffer for wrapped message
    size_t wrapped_size = data_len + 128;  // Extra space for metadata
    char *wrapped = ws_buffer_pool_alloc(wrapped_size);

    if (wrapped == NULL) {
        ESP_LOGE(TAG, "Failed to allocate WebSocket message buffer");
        return;
    }

    // Wrap with metadata
    int64_t timestamp = esp_timer_get_time() / 1000;
    int written = snprintf(wrapped, wrapped_size,
                          "{\"timestamp\":%lld,\"data\":%s}",
                          timestamp, telemetry_data);

    if (written > 0 && (size_t)written < wrapped_size) {
        ESP_LOGI(TAG, "WebSocket message ready: %s", wrapped);
        // Would call ws_client_list_broadcast() here...
    }

    ws_buffer_pool_free(wrapped);

    // Show statistics after operation
    ws_buffer_pool_stats_t stats;
    ws_buffer_pool_get_stats(&stats);
    ESP_LOGI(TAG, "After message: %u/%u buffers in use",
             stats.current_usage, 8);
}
