import requests
import time
import sys
import json

def test_model(model_name, prompt, timeout=60):
    """Test a model and measure its response time."""
    url = "http://localhost:11434/api/generate"
    
    payload = {
        "model": model_name,
        "prompt": prompt,
        "stream": False
    }
    
    print(f"\nTesting model: {model_name}")
    print(f"Prompt: {prompt}")
    start_time = time.time()
    
    try:
        response = requests.post(url, json=payload, timeout=timeout)
        elapsed_time = time.time() - start_time
        
        print(f"Request completed in {elapsed_time:.2f} seconds")
        print(f"Status code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            response_text = result.get('response', 'No response content')
            print(f"Response length: {len(response_text)} characters")
            print("First 100 characters of response:")
            print(response_text[:100] + "..." if len(response_text) > 100 else response_text)
            return True, elapsed_time
        else:
            print(f"Error: Received status code {response.status_code}")
            print(response.text)
            return False, elapsed_time
    except requests.exceptions.Timeout:
        elapsed_time = time.time() - start_time
        print(f"Request timed out after {elapsed_time:.2f} seconds")
        return False, elapsed_time
    except Exception as e:
        print(f"Error: {e}")
        return False, 0

def compare_models(models, prompts, timeout=60):
    """Compare the performance of multiple models on multiple prompts."""
    results = {}
    
    for model in models:
        results[model] = []
        
        for prompt in prompts:
            success, time_taken = test_model(model, prompt, timeout)
            results[model].append({
                "prompt": prompt,
                "success": success,
                "time": time_taken
            })
    
    # Print comparison summary
    print("\n" + "="*50)
    print("PERFORMANCE COMPARISON")
    print("="*50)
    
    for model in models:
        total_time = sum(result["time"] for result in results[model] if result["success"])
        success_count = sum(1 for result in results[model] if result["success"])
        
        print(f"\nModel: {model}")
        print(f"Successful requests: {success_count}/{len(prompts)}")
        
        if success_count > 0:
            avg_time = total_time / success_count
            print(f"Average response time: {avg_time:.2f} seconds")
        else:
            print("Average response time: N/A (no successful requests)")
    
    return results

if __name__ == "__main__":
    models = ["deepseek-coder", "deepseek-coder-optimized"]
    prompts = [
        "Write a simple function to add two numbers in Python",
        "Explain what a closure is in JavaScript"
    ]
    timeout = 60  # 60 seconds timeout
    
    print("Comparing performance between original and optimized models...")
    compare_models(models, prompts, timeout)
