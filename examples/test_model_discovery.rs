//! Test the new model discovery API from NLP engine

use nodespace_nlp_engine::LocalNLPEngine;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🔍 Testing Model Discovery API");
    println!("===============================\n");

    // Test 1: List available models
    println!("1️⃣ Testing available models discovery:");
    match LocalNLPEngine::list_available_models() {
        Ok(models) => {
            println!("   ✅ Found {} available models", models.len());
            for model in &models {
                println!("   📦 Model: {:?}", model);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list models: {}", e);
        }
    }

    // Test 2: Get model info
    println!("\n2️⃣ Testing model information retrieval:");
    match LocalNLPEngine::list_available_models() {
        Ok(models) => {
            for model in models.iter().take(2) {
                // Test first 2 models
                match LocalNLPEngine::get_model_info(model) {
                    Ok(info) => {
                        println!("   ✅ Model info for {:?}:", model);
                        println!("      Name: {}", info.name);
                        println!("      Path: {:?}", info.path);
                        println!("      Available: {}", info.available);
                    }
                    Err(e) => {
                        println!("   ❌ Failed to get info for {:?}: {}", model, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list models: {}", e);
        }
    }

    // Test 3: Validate model availability
    println!("\n3️⃣ Testing model availability validation:");
    match LocalNLPEngine::list_available_models() {
        Ok(models) => {
            for model in models.iter().take(2) {
                // Test first 2 models
                match LocalNLPEngine::validate_model_availability(model) {
                    Ok(available) => {
                        if available {
                            println!("   ✅ Model {:?} is available", model);
                        } else {
                            println!("   ⚠️  Model {:?} is not available (files missing)", model);
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Failed to validate {:?}: {}", model, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list models: {}", e);
        }
    }

    println!("\n🎉 Model Discovery API Testing Completed!");
    println!("==========================================");

    Ok(())
}
