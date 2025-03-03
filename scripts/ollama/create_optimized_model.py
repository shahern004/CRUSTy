import requests
import json
import time
import sys

def create_optimized_model(base_model, new_model_name):
    """Create an optimized version of an existing Ollama model."""
    url = "http://localhost:11434/api/create"
    
    # Modelfile content with optimized parameters
    modelfile = f"""
FROM {base_model}

# Performance optimization parameters
PARAMETER num_ctx 2048
PARAMETER num_thread 8
PARAMETER num_gpu 50
PARAMETER temperature 0.7
PARAMETER top_k 40
PARAMETER top_p 0.9
PARAMETER repeat_penalty 1.1
"""
    
    payload = {
        "name": new_model_name,
        "modelfile": modelfile
    }
    
    print(f"Creating optimized model '{new_model_name}' based on '{base_model}'...")
    print("Modelfile content:")
    print(modelfile)
    
    try:
        response = requests.post(url, json=payload)
        
        if response.status_code == 200:
            print(f"\nModel '{new_model_name}' created successfully!")
            return True
        else:
            print(f"Error: Received status code {response.status_code}")
            print(response.text)
            return False
    except Exception as e:
        print(f"Error creating model: {e}")
        return False

if __name__ == "__main__":
    base_model = "deepseek-coder"
    new_model_name = "deepseek-coder-optimized"
    
    print("This script will create an optimized version of the deepseek-coder model.")
    print("The optimized model will have reduced context size and other performance tweaks.")
    print(f"Base model: {base_model}")
    print(f"New model name: {new_model_name}")
    
    create_optimized_model(base_model, new_model_name)
