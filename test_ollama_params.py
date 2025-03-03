import requests
import time
import sys
import json

def test_ollama_with_params(model_name, prompt, timeout=60):
    """Test Ollama API with specific parameters to improve performance."""
    url = "http://localhost:11434/api/generate"
    
    payload = {
        "model": model_name,
        "prompt": prompt,
        "stream": False,
        "options": {
            "num_ctx": 2048,       # Reduced context size
            "num_thread": 8,       # Use 8 threads (matching CPU cores)
            "temperature": 0.7,    # Standard temperature
            "top_k": 40,           # Standard top_k
            "top_p": 0.9           # Standard top_p
        }
    }
    
    print(f"Sending request to {model_name} with custom parameters...")
    print(f"Parameters: {json.dumps(payload['options'], indent=2)}")
    start_time = time.time()
    
    try:
        response = requests.post(url, json=payload, timeout=timeout)
        elapsed_time = time.time() - start_time
        
        print(f"Request completed in {elapsed_time:.2f} seconds")
        print(f"Status code: {response.status_code}")
        
        if response.status_code == 200:
            result = response.json()
            print("\nResponse:")
            print(result.get('response', 'No response content'))
            return True
        else:
            print(f"Error: Received status code {response.status_code}")
            print(response.text)
            return False
    except requests.exceptions.Timeout:
        elapsed_time = time.time() - start_time
        print(f"Request timed out after {elapsed_time:.2f} seconds")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    model = "deepseek-coder"
    prompt = "Write a simple function to add two numbers in Python"
    timeout = 60  # 60 seconds timeout
    
    print(f"Testing {model} with custom parameters and a {timeout}s timeout...")
    test_ollama_with_params(model, prompt, timeout)
