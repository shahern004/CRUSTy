import requests
import json

def test_ollama_model(model_name):
    """Test if an Ollama model is working by sending a simple request."""
    url = "http://localhost:11434/api/generate"
    
    payload = {
        "model": model_name,
        "prompt": "Write a simple 'Hello, World!' program in Python",
        "stream": False
    }
    
    try:
        response = requests.post(url, json=payload)
        if response.status_code == 200:
            result = response.json()
            print(f"Model {model_name} is working!")
            print("\nResponse:")
            print(result.get('response', 'No response content'))
            return True
        else:
            print(f"Error: Received status code {response.status_code}")
            print(response.text)
            return False
    except Exception as e:
        print(f"Error connecting to Ollama: {e}")
        return False

if __name__ == "__main__":
    print("Testing deepseek-coder model...")
    test_ollama_model("deepseek-coder")
