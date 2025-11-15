// Wasmtime test harness for JWT business logic component
// This demonstrates that the component works WITHOUT Spin!

use anyhow::Result;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

wasmtime::component::bindgen!({
    path: "../../components/business-logic/wit",
    world: "business-logic",
    async: false
});

struct MyState {
    wasi: WasiCtx,
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&mut self) -> &mut ResourceTable {
        unimplemented!()
    }
}

fn main() -> Result<()> {
    println!("🧪 Testing JWT Component with Wasmtime (NO Spin!)\n");
    println!("=" .repeat(60));

    // Setup Wasmtime engine with component model support
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    // Create WASI context (component needs WASI for clocks/random)
    let wasi = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(&engine, MyState { wasi });

    // Load the component
    println!("\n📦 Loading component: toyota_business_logic.wasm");
    let component = Component::from_file(
        &engine,
        "../../target/wasm32-wasip1/release/toyota_business_logic.wasm",
    )?;
    println!("✅ Component loaded successfully!");

    // Instantiate with WASI linker
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;

    println!("\n🔗 Instantiating component...");
    let (bindings, _instance) = BusinessLogic::instantiate(&mut store, &component, &linker)?;
    println!("✅ Component instantiated!");

    let jwt = bindings.toyota_business_logic_jwt();
    let test_secret = b"test-jwt-secret-key-for-poc".to_vec();
    let test_hmac = b"test-hmac-key-for-poc".to_vec();

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 1: Generate Access Token");
    println!("=" .repeat(60));

    let access_token = jwt
        .call_generate_access_token(&mut store, "testuser", &test_secret)?
        .map_err(|e| anyhow::anyhow!("Failed to generate access token: {}", e))?;

    println!("✅ Access token generated!");
    println!("   Token length: {} chars", access_token.len());
    println!("   Token preview: {}...", &access_token[0..50.min(access_token.len())]);

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 2: Verify Access Token");
    println!("=" .repeat(60));

    let claims = jwt
        .call_verify_token(&mut store, &access_token, &test_secret)?
        .map_err(|e| anyhow::anyhow!("Failed to verify token: {}", e))?;

    println!("✅ Token verified successfully!");
    println!("   Subject (username): {}", claims.sub);
    println!("   Token type: {}", claims.token_type);
    println!("   Issued at: {}", claims.iat);
    println!("   Expires at: {}", claims.exp);
    println!("   JWT ID: {}", claims.jti);

    assert_eq!(claims.sub, "testuser");
    assert_eq!(claims.token_type, "access");

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 3: Generate Refresh Token");
    println!("=" .repeat(60));

    let refresh_token = jwt
        .call_generate_refresh_token(&mut store, "testuser", &test_secret)?
        .map_err(|e| anyhow::anyhow!("Failed to generate refresh token: {}", e))?;

    println!("✅ Refresh token generated!");
    println!("   Token length: {} chars", refresh_token.len());

    let refresh_claims = jwt
        .call_verify_token(&mut store, &refresh_token, &test_secret)?
        .map_err(|e| anyhow::anyhow!("Failed to verify refresh token: {}", e))?;

    assert_eq!(refresh_claims.token_type, "refresh");
    println!("✅ Refresh token verified!");
    println!("   Token type: {}", refresh_claims.token_type);

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 4: Token Security - Wrong Secret");
    println!("=" .repeat(60));

    let wrong_secret = b"wrong-secret".to_vec();
    let result = jwt.call_verify_token(&mut store, &access_token, &wrong_secret)?;

    match result {
        Err(e) => {
            println!("✅ Correctly rejected token with wrong secret!");
            println!("   Error: {}", e);
        }
        Ok(_) => {
            anyhow::bail!("❌ SECURITY FAILURE: Token verified with wrong secret!");
        }
    }

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 5: Hash Username");
    println!("=" .repeat(60));

    let hash1 = jwt.call_hash_username(&mut store, "testuser", &test_hmac)?;
    let hash2 = jwt.call_hash_username(&mut store, "testuser", &test_hmac)?;
    let hash3 = jwt.call_hash_username(&mut store, "different", &test_hmac)?;

    println!("✅ Username hashing works!");
    println!("   Hash length: {} chars", hash1.len());
    println!("   Hash: {}", hash1);

    assert_eq!(hash1, hash2, "Same username should produce same hash");
    assert_ne!(hash1, hash3, "Different usernames should produce different hashes");

    println!("   ✓ Consistency check passed");
    println!("   ✓ Uniqueness check passed");

    println!("\n" + &"=".repeat(60));
    println!("🧪 TEST 6: Invalid Token Format");
    println!("=" .repeat(60));

    let invalid_token = "not.a.valid.jwt.token";
    let result = jwt.call_verify_token(&mut store, invalid_token, &test_secret)?;

    match result {
        Err(e) => {
            println!("✅ Correctly rejected invalid token format!");
            println!("   Error: {}", e);
        }
        Ok(_) => {
            anyhow::bail!("❌ ERROR: Invalid token was accepted!");
        }
    }

    println!("\n" + &"=".repeat(60));
    println!("✅ ALL TESTS PASSED!");
    println!("=" .repeat(60));
    println!("\n🎉 SUCCESS: JWT component works standalone with wasmtime!");
    println!("💡 This component has ZERO Spin dependencies");
    println!("🚀 It can be composed with other components or used in any WASI runtime");
    println!();

    Ok(())
}
