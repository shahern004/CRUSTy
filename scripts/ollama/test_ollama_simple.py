import requests
import sys

def check_ollama_api():
    """Check if the Ollama API is responding."""
    try:
        response = requests.post("http://localhost:11434/api/health")
        print(f"API Health Check Status Code: {response.status_code}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            print("Ollama API is responding correctly!")
            return True
        else:
            print(f"Ollama API returned non-200 status code: {response.status_code}")
            return False
    except Exception as e:
        print(f"Error connecting to Ollama API: {e}")
        return False

def check_model_exists(model_name):
    """Check if a specific model exists in Ollama."""
    try:
        response = requests.get("http://localhost:11434/api/tags")
        if response.status_code == 200:
            models = response.json().get("models", [])
            for model in models:
                if model.get("name") == model_name:
                    print(f"Model '{model_name}' exists in Ollama!")
                    return True
            print(f"Model '{model_name}' not found in Ollama.")
            return False
        else:
            print(f"Error getting model list: {response.status_code}")
            return False
    except Exception as e:
        print(f"Error checking model existence: {e}")
        return False

if __name__ == "__main__":
    print("Checking Ollama API...")
    if check_ollama_api():
        print("\nChecking if deepseek-coder model exists...")
        check_model_exists("deepseek-coder")
    else:
        print("Ollama API is not responding. Please check if Ollama is running.")
        sys.exit(1)
