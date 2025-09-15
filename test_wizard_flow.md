# Improved Manx Wizard Flow

## Key Improvements Made:

### 1. **Simplified Questions**
- Removed repetitive provider selection menus
- Focused on core user decisions: Context7, Search Engine, AI Features
- Each step has clear value propositions

### 2. **Back Navigation**
- Added navigation system with Back/Continue/Skip/Quit options
- Users can go back to previous steps to change decisions
- Clean navigation flow between all steps

### 3. **Better Embedding Setup**
- Simplified to 2 main choices: Hash (fast) vs Neural (better)
- Automatic download of recommended model (all-MiniLM-L6-v2)
- Uses actual manx embedding configuration system
- Proper error handling with fallback to hash search

### 4. **Realistic AI Setup**
- Defaults to "Skip AI" - don't assume users want it
- Only shows popular providers (OpenAI, Anthropic, Groq)
- Uses cost-effective model defaults (gpt-4o-mini, claude-haiku)
- Clear value proposition for AI features

### 5. **Configuration Testing**
- Tests actual manx configuration values
- Validates API keys with proper formats
- Tests embedding model setup
- Clear success/failure feedback

## Sample Wizard Flow:

```
🚀 Welcome to Manx Setup Wizard!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Let's configure manx for optimal documentation search.

[Step 1/4] Context7 API
───────────────────────
🚀 Context7 provides access to official documentation from thousands of projects.
It's optional but highly recommended for the best search experience.

Without Context7:
  • Limited to hash-based search
  • No official documentation access
With Context7:
  • Search official docs from React, Python, Rust, etc.
  • Access to latest documentation
  • Better search results

? How would you like to proceed? ❯ Get a free API key (opens browser)
  Skip (I'll set it up later)
  I have an API key

[Opens browser to https://context7.com/]
? Paste your Context7 API key: █

✓ Context7 API configured!

? What would you like to do?
❯ Continue
  Back
  Quit setup

[Step 2/4] Search Engine
────────────────────────
🔍 Choose your search engine - this affects how well manx understands your searches:

Hash Search (Default):
  ✓ Works immediately, no setup required
  ✓ Fast and reliable
  ✓ Perfect for exact keyword matching

Neural Search (Recommended):
  ✓ Understands meaning: "auth" finds "authentication"
  ✓ Better results for complex queries
  ✓ Small download (~22MB), runs locally

? Select search engine
  Hash Search (fast, no download)
❯ Neural Search (download small model ~22MB)
  Keep current setting

Downloading all-MiniLM-L6-v2 model...
⠙ Downloading all-MiniLM-L6-v2 (~22MB)...
✓ Neural search model installed!

✓ Neural search ready!
  Searches will now understand context and meaning.

? What would you like to do?
❯ Continue
  Back
  Skip this step
  Quit setup

[Step 3/4] AI Features
─────────────────────
🤖 Enable AI features for comprehensive answers with explanations?

AI features provide:
  • Detailed explanations with code examples
  • Answers synthesized from multiple sources
  • Source citations for verification

Without AI:
  • Raw documentation snippets (still very useful!)
  • Faster responses, no API costs

? Add AI features?
❯ Skip AI features (use basic search only)
  OpenAI (GPT models) - most popular
  Anthropic (Claude models) - this tool's creator
  Groq (fastest inference)
  I'll set this up later

AI features skipped - manx will work great without them!
💡 You can enable AI later with: manx config --openai-api <key>

? What would you like to do?
❯ Continue
  Back
  Skip this step
  Quit setup

[Step 4/4] Summary
─────────────────
🎯 Your manx is configured!

Configuration Summary:

  ✓ Context7 API for official documentation
  ✓ Neural search engine: all-MiniLM-L6-v2 (semantic understanding)
  ○ AI features (raw docs only - still very useful!)

────────────────────────────────────────
? Test your configuration now? (y/N) n

Try these commands:
  manx snippet react hooks
  manx search "python async patterns"
  manx doc fastapi

💡 Your config is saved to ~/.config/manx/config.json

? What would you like to do?
❯ Continue
  Back
  Quit setup

──────────────────────────────────────────────────
🎉 Setup complete! manx is ready!

Get started with these commands:
  manx snippet react hooks
  manx search "authentication patterns"
  manx doc fastapi middleware

📚 Need help? Try: manx --help
```

## Technical Improvements:

1. **Navigation System**: Added `WizardAction` and `WizardStep` enums for proper flow control
2. **Proper Config Integration**: Uses actual manx config structs and saves properly
3. **Embedding Download**: Uses `OnnxProvider::download_model()` from manx's existing system
4. **Error Handling**: Graceful fallback to hash search if neural model download fails
5. **Validation**: Proper API key format validation for each provider
6. **Testing**: Real configuration testing with spinners and feedback

This wizard now properly integrates with manx's actual configuration system and provides a much better user experience!