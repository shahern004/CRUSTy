import requests
import json
import sys
import time

def stream_from_ollama(model_name, prompt):
    """Stream response from Ollama model to avoid loading loop issues."""
    url = "http://localhost:11434/api/generate"
    
    payload = {
        "model": model_name,
        "prompt": prompt,
        "stream": True  # Enable streaming
    }
    
    print(f"Streaming response from {model_name}...")
    print(f"Prompt: {prompt}")
    print("\nResponse:")
    
    start_time = time.time()
    total_tokens = 0
    
    try:
        # Send request and process streaming response
        with requests.post(url, json=payload, stream=True) as response:
            if response.status_code != 200:
                print(f"Error: Received status code {response.status_code}")
                print(response.text)
                return False
            
            # Process each chunk as it arrives
            for line in response.iter_lines():
                if line:
                    # Parse JSON from the line
                    chunk = json.loads(line)
                    
                    # Extract and print the response token
                    if 'response' in chunk:
                        token = chunk['response']
                        print(token, end='', flush=True)
                        total_tokens += 1
                    
                    # Check if done
                    if chunk.get('done', False):
                        break
        
        elapsed_time = time.time() - start_time
        print(f"\n\nStreaming completed in {elapsed_time:.2f} seconds")
        print(f"Total tokens: {total_tokens}")
        print(f"Tokens per second: {total_tokens / elapsed_time:.2f}")
        return True
    
    except Exception as e:
        print(f"\nError: {e}")
        return False

if __name__ == "__main__":
    # Use the optimized model by default
    model = "deepseek-coder-optimized"
    prompt = "Write a simple function to add two numbers in Python"
    
    print("This script demonstrates streaming responses from Ollama")
    print("Streaming allows you to see results immediately as they're generated")
    print("This helps avoid the 'loading loop' issue\n")
    
    stream_from_ollama(model, prompt)
