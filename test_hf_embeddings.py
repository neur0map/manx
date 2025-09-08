#!/usr/bin/env python3
"""
Test script to verify HuggingFace embeddings work properly
This simulates what our ONNX implementation should do
"""

import requests
import json
import numpy as np

def test_hf_embeddings():
    # Test texts for comparison
    texts = [
        "React hooks useState for state management",
        "useState React hook manages local component state", 
        "Python Django models for database operations",
        "JavaScript async await for asynchronous programming"
    ]
    
    # HuggingFace API endpoint (no API key needed for public models)
    model = "sentence-transformers/all-MiniLM-L6-v2"
    api_url = f"https://api-inference.huggingface.co/models/{model}"
    
    headers = {"Content-Type": "application/json"}
    
    print(f"🧪 Testing HuggingFace embeddings with model: {model}")
    print("=" * 60)
    
    embeddings = []
    
    for i, text in enumerate(texts):
        print(f"🔄 Processing text {i+1}: {text[:50]}...")
        
        payload = {
            "inputs": text,
            "options": {"wait_for_model": True}
        }
        
        try:
            response = requests.post(api_url, headers=headers, json=payload, timeout=30)
            
            if response.status_code == 200:
                embedding = response.json()
                embeddings.append(embedding)
                print(f"   ✅ Got {len(embedding)}D embedding")
            else:
                print(f"   ❌ Error: {response.status_code} - {response.text}")
                
        except Exception as e:
            print(f"   ❌ Exception: {e}")
    
    if len(embeddings) >= 4:
        print("\n📊 Computing similarity scores...")
        
        # Calculate cosine similarity
        def cosine_similarity(a, b):
            a, b = np.array(a), np.array(b)
            return np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b))
        
        # Test semantic understanding
        react_1 = embeddings[0]  # "React hooks useState"
        react_2 = embeddings[1]  # "useState React hook"
        python_django = embeddings[2]  # "Python Django"
        js_async = embeddings[3]  # "JavaScript async"
        
        sim_react = cosine_similarity(react_1, react_2)
        sim_cross1 = cosine_similarity(react_1, python_django)
        sim_cross2 = cosine_similarity(react_1, js_async)
        
        print(f"   React texts similarity: {sim_react:.3f}")
        print(f"   React vs Python similarity: {sim_cross1:.3f}")
        print(f"   React vs JS async similarity: {sim_cross2:.3f}")
        
        print("\n🎯 Expected Results:")
        print("   - React texts should have HIGH similarity (>0.8)")
        print("   - Cross-topic similarities should be LOWER (<0.6)")
        print("   - This demonstrates semantic understanding")
        
        if sim_react > 0.8:
            print("   ✅ Semantic understanding WORKING!")
        else:
            print("   ⚠️ Semantic understanding may need improvement")

if __name__ == "__main__":
    test_hf_embeddings()