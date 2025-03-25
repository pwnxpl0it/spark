#!/bin/bash
# This script is just for testing a few use cases before pushing to repo
# I will push unit tests when I finish writing them..

# Color definitions
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
RED="\033[0;31m"
NC="\033[0m" # No Color

# Check if file or directory exists
check_exists() {
    if [[ -e $1 ]]; then
        echo -e "${GREEN}[+] $1 exists${NC}"
    else
        echo -e "${RED}[-] $1 does not exist${NC}"
    fi
}

# Backup current templates and configuration
backup_templates() {
    echo -e "${YELLOW}[*] Backing up current templates...${NC}"
    mv ~/.config/spark ~/.config/spark_bak
}

# Restore old templates and configuration
restore_templates() {
    echo -e "${YELLOW}[*] Restoring old templates...${NC}"
    mv ~/.config/spark_bak ~/.config/spark
}

# Recompile Spark
recompile_spark() {
    echo -e "${YELLOW}[*] Recompiling Spark...${NC}"
    cargo install --path . --offline
}

# Initialize configuration
initialize_config() {
    echo -e "${YELLOW}[*] Testing Config initialization...${NC}"
    spark
}

# Test "new" template
test_new_template() {
    echo -e "${YELLOW}[*] Testing 'new' template...${NC}"
	sleep 5
    spark new
}

# Check config files
check_config_files() {
    echo -e "${YELLOW}[*] Checking config and template files...${NC}"
    check_exists "$HOME/.config/spark/config.toml"
    check_exists "$HOME/.config/spark/templates/"
}

# Clean up initialized files
cleanup() {
    echo -e "${YELLOW}[*] Cleaning up initialized files...${NC}"
    rm -rf ~/.config/spark
}

# Run all steps
main() {
    backup_templates
    recompile_spark
    initialize_config
    test_new_template
    check_config_files
	echo -e "${RED} Sleeping for 5 seconds before restoring templates...${NC}"
	sleep 5
    cleanup
    restore_templates
    echo -e "${GREEN}[+] All tests completed.${NC}"
}

# Execute the main function
main
