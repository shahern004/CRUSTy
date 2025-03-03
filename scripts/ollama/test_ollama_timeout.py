import requests
import time
import sys

def test_ollama_with_timeout(model_name, prompt, timeout=30):
    """Test Ollama API with a timeout to prevent hanging."""
    url = "http://localhost:11434/api/generate"
    
    payload = {
        "model": model_name,
        "prompt": prompt,
        "stream": False
    }
    
    print(f"Sending request to {model_name} with {timeout}s timeout...")
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
    timeout = 15  # 15 seconds timeout
    
    print(f"Testing {model} with a {timeout}s timeout...")
    test_ollama_with_timeout(model, prompt, timeout)
