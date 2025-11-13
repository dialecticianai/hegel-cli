#!/usr/bin/env bash
# Reproduce bug: hegel next at vision node jumps to execution workflow
# Expected: vision -> architecture (intra-workflow transition)
# Actual: vision -> execution:plan (inter-workflow transition)

set -e

# Create temp directory for testing
TEST_DIR=$(mktemp -d)
echo "🧪 Test directory: $TEST_DIR"
cd "$TEST_DIR"

# Setup: Create a minimal existing project (retrofit scenario)
echo "📝 Setting up retrofit project..."
mkdir -p src
cat > src/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

cat > Cargo.toml << 'EOF'
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
EOF

git init > /dev/null
git add . > /dev/null
git commit -m "Initial commit" > /dev/null

echo "✅ Retrofit project created"
echo ""

# Initialize .hegel directory
mkdir -p .hegel
echo "✅ .hegel directory created"
echo ""

# Step 1: Start init-retrofit workflow
echo "🚀 Step 1: Starting init-retrofit workflow..."
hegel start init-retrofit 2>&1 | head -5
echo ""

# Step 2: Advance through detect_existing
echo "⏭️  Step 2: Advancing from detect_existing..."
hegel next 2>&1 | grep -E "Transitioned|Mode:" || true
echo ""

# Step 3: Advance through code_map
echo "⏭️  Step 3: Advancing from code_map..."
hegel next 2>&1 | grep -E "Transitioned|Mode:" || true
echo ""

# Step 4: Advance through customize_claude
echo "⏭️  Step 4: Advancing from customize_claude..."
hegel next 2>&1 | grep -E "Transitioned|Mode:" || true
echo ""

# Step 5: Check status before the problematic transition
echo "📊 Step 5: Status before problematic transition..."
hegel status 2>&1 | grep -E "Meta-mode|Current workflow|Current node"
echo ""

# Step 5.5: Run hegel > HEGEL.md (as user did in original session)
echo "📄 Step 5.5: Running 'hegel > HEGEL.md' (as in original session)..."
hegel > HEGEL.md
echo "✅ HEGEL.md created ($(wc -l < HEGEL.md) lines)"
echo ""

# Step 6: THE BUG - Advance from vision (should go to architecture, not execution)
echo "🐛 Step 6: THE BUG - Advancing from vision node..."
echo "Expected: Transitioned: vision → architecture (Mode: init)"
echo "Actual:"
hegel next 2>&1 | grep -E "Transitioned|Mode:" || true
echo ""

# Step 7: Check final status to confirm bug
echo "📊 Step 7: Final status (should be at architecture in init mode)..."
hegel status 2>&1 | grep -E "Meta-mode|Current workflow|Current node"
echo ""

# Cleanup
cd - > /dev/null
echo "🧹 Cleanup: Test directory preserved at $TEST_DIR for inspection"
echo "   To delete: rm -rf $TEST_DIR"
