import ollama

response = ollama.chat(
    model="qwen3.6",
    messages=[
        {
            "role": "user",
            "content": "Why is the sky blue?",
        },
    ],
)
print(response["message"]["content"])
