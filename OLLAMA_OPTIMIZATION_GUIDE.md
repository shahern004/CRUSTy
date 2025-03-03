# Ollama Performance Optimization Guide

This guide provides solutions for optimizing Ollama performance and addressing the "API request loading loop" issue with the deepseek-coder model.

## Understanding the Issue

The "API request loading loop" occurs when Ollama takes an unusually long time to process requests, making it appear as if the request is hanging. This can happen due to:

1. Resource constraints (CPU, memory)
2. Large model size and complexity
3. Default configuration parameters that aren't optimized for your system
4. Long context windows that require more processing time

## Solution: Optimized Model

We've created an optimized version of the deepseek-coder model with the following improvements:

- Reduced context size (2048 tokens instead of the default 16384)
- Optimized thread usage to match your CPU cores (8 threads)
- Adjusted GPU utilization and other parameters for better performance

### Using the Optimized Model

To use the optimized model, simply replace `deepseek-coder` with `deepseek-coder-optimized` in your commands:

```bash
# Instead of
ollama run deepseek-coder "your prompt here"

# Use
ollama run deepseek-coder-optimized "your prompt here"
```

## Additional Optimization Tips

### 1. Adjust System Resources

Ensure Ollama has sufficient resources:

- Close other resource-intensive applications
- Consider increasing swap space if RAM is limited
- If using a GPU, ensure drivers are up-to-date

### 2. Use Shorter Prompts

Shorter prompts require less processing time. Try to be concise and specific in your requests.

### 3. Stream Responses

When using the API, set `stream: true` to get tokens as they're generated rather than waiting for the entire response:

```python
payload = {
    "model": "deepseek-coder-optimized",
    "prompt": "your prompt",
    "stream": true
}
```

### 4. Custom Model Parameters

You can create custom models with different parameters using the `create_optimized_model.py` script:

```bash
python create_optimized_model.py
```

Adjust the parameters in the script to find the optimal configuration for your system.

### 5. Monitor Performance

Use the `compare_models.py` script to compare performance between different models:

```bash
python compare_models.py
```

## Troubleshooting

### If Requests Still Timeout

1. Try increasing the timeout value in your API requests
2. Further reduce the context size (e.g., to 1024 or 512)
3. Consider using a smaller model if available

### If You See Warnings in Logs

The warning `key not found key=llama.attention.key_length default=128` is non-critical and indicates a configuration parameter wasn't explicitly set. It uses the default value and doesn't affect functionality.

## Scripts Included

1. `create_optimized_model.py` - Creates an optimized version of the deepseek-coder model
2. `compare_models.py` - Compares performance between original and optimized models
3. `test_ollama_params.py` - Tests the model with custom parameters
4. `test_ollama_timeout.py` - Tests the model with a timeout to prevent hanging

## System Requirements

For optimal performance with the deepseek-coder model:

- 8+ CPU cores (AMD Ryzen 7 5800X or equivalent)
- 16+ GB RAM
- GPU with 4+ GB VRAM (optional but recommended)
