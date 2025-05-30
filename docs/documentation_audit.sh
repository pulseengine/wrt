#!/bin/bash
# Documentation audit script for WRT codebase
# This script checks all crates for documentation completeness

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Header
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}WRT CRATE DOCUMENTATION AUDIT${NC}"
echo -e "${YELLOW}========================================${NC}"
echo ""

# List of all crates
CRATES=(
  "wrt"
  "wrt-error"
  "wrt-sync"
  "wrt-format"
  "wrt-foundation"
  "wrt-decoder"
  "wrt-runtime"
  "wrt-logging"
  "wrt-instructions"
  "wrt-component"
  "wrt-host"
  "wrt-intercept"
  "wrt-test-registry"
  "wrtd"
  "xtask"
  "wrt-verification-tool"
)

check_file_exists() {
  local crate=$1
  local file=$2
  
  if [ -f "$crate/$file" ]; then
    echo -e "${GREEN}✓${NC} $file exists"
    return 0
  else
    echo -e "${RED}✗${NC} $file missing"
    return 1
  fi
}

check_cargo_metadata() {
  local crate=$1
  local metadata_items=("description" "documentation" "keywords" "categories")
  local has_all_metadata=true
  
  echo -e "\nChecking Cargo.toml metadata:"
  
  for item in "${metadata_items[@]}"; do
    if grep -q "$item =" "$crate/Cargo.toml"; then
      echo -e "${GREEN}✓${NC} $item"
    else
      echo -e "${RED}✗${NC} $item missing"
      has_all_metadata=false
    fi
  done
  
  if grep -q "package.metadata.docs.rs" "$crate/Cargo.toml"; then
    echo -e "${GREEN}✓${NC} docs.rs configuration"
  else
    echo -e "${RED}✗${NC} docs.rs configuration missing"
    has_all_metadata=false
  fi
  
  return $has_all_metadata
}

check_crate_doc_quality() {
  local crate=$1
  local lib_file="$crate/src/lib.rs"
  local has_good_docs=true
  
  echo -e "\nChecking lib.rs documentation quality:"
  
  if [ -f "$lib_file" ]; then
    # Check for crate-level documentation
    if grep -q "^//! " "$lib_file"; then
      echo -e "${GREEN}✓${NC} Has crate-level documentation"
      
      # Check for example in crate docs
      if grep -q "^//! \`\`\`rust" "$lib_file"; then
        echo -e "${GREEN}✓${NC} Documentation includes examples"
      else
        echo -e "${YELLOW}⚠${NC} No examples in crate documentation"
        has_good_docs=false
      fi
      
      # Check for feature section
      if grep -q "^//! ## Features" "$lib_file"; then
        echo -e "${GREEN}✓${NC} Documentation includes features section"
      else
        echo -e "${YELLOW}⚠${NC} No features section in documentation"
        has_good_docs=false
      fi
    else
      echo -e "${RED}✗${NC} Missing crate-level documentation"
      has_good_docs=false
    fi
    
    # Check for missing_docs lint
    if grep -q "#!\[warn(missing_docs)]" "$lib_file"; then
      echo -e "${GREEN}✓${NC} Has #![warn(missing_docs)] lint"
    else
      echo -e "${RED}✗${NC} Missing #![warn(missing_docs)] lint"
      has_good_docs=false
    fi
    
    # Check for docsrs configuration
    if grep -q "#!\[cfg_attr(docsrs, " "$lib_file"; then
      echo -e "${GREEN}✓${NC} Has docsrs configuration"
    else
      echo -e "${YELLOW}⚠${NC} Missing docsrs configuration"
      has_good_docs=false
    fi
  else
    echo -e "${RED}✗${NC} lib.rs file not found"
    has_good_docs=false
  fi
  
  return $has_good_docs
}

# Perform the audit
echo -e "Found ${#CRATES[@]} crates to audit\n"

MISSING_README=()
MISSING_METADATA=()
POOR_DOCUMENTATION=()

for crate in "${CRATES[@]}"; do
  echo -e "${YELLOW}Checking $crate...${NC}"
  
  # Check for README.md
  if ! check_file_exists "$crate" "README.md"; then
    MISSING_README+=("$crate")
  fi
  
  # Check Cargo.toml metadata
  if ! check_cargo_metadata "$crate"; then
    MISSING_METADATA+=("$crate")
  fi
  
  # Check documentation quality
  if ! check_crate_doc_quality "$crate"; then
    POOR_DOCUMENTATION+=("$crate")
  fi
  
  echo -e "----------------------------------------"
done

# Summary
echo -e "\n${YELLOW}SUMMARY:${NC}"
echo -e "Total crates: ${#CRATES[@]}"
echo -e "Crates missing README: ${#MISSING_README[@]}"
echo -e "Crates with incomplete metadata: ${#MISSING_METADATA[@]}"
echo -e "Crates with poor documentation: ${#POOR_DOCUMENTATION[@]}"

if [ ${#MISSING_README[@]} -gt 0 ]; then
  echo -e "\n${RED}Crates missing README:${NC}"
  for crate in "${MISSING_README[@]}"; do
    echo "- $crate"
  done
fi

if [ ${#MISSING_METADATA[@]} -gt 0 ]; then
  echo -e "\n${RED}Crates with incomplete metadata:${NC}"
  for crate in "${MISSING_METADATA[@]}"; do
    echo "- $crate"
  done
fi

if [ ${#POOR_DOCUMENTATION[@]} -gt 0 ]; then
  echo -e "\n${RED}Crates with poor documentation:${NC}"
  for crate in "${POOR_DOCUMENTATION[@]}"; do
    echo "- $crate"
  done
fi

echo -e "\n${YELLOW}RECOMMENDATION:${NC}"
echo "1. Create missing README.md files"
echo "2. Update Cargo.toml metadata for identified crates"
echo "3. Improve crate-level documentation in lib.rs files"
echo "4. Add usage examples to all public APIs"
echo -e "5. Run clippy with ${GREEN}cargo clippy --all-targets --all-features -- -W clippy::missing_docs_in_private_items${NC}"

echo -e "\nSee templates/crate_template/README.md.template for documentation standards and templates" 