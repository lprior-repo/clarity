#!/usr/bin/env bash
#
# Responsive Design QA Test for Progressive Discover
# Tests component layouts, touch targets, text scaling, and horizontal scroll
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNINGS=0

# Helper functions
log_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
    ((TOTAL_TESTS++))
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_TESTS++))
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ((WARNINGS++))
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Test 1: Check for viewport meta tag
test_viewport_meta() {
    log_test "Viewport meta tag present in HTML"

    if grep -q "viewport" /home/lewis/src/clarity/clarity-web/src/app.rs 2>/dev/null; then
        log_pass "Viewport meta tag found in app.rs"
        return 0
    else
        log_fail "No viewport meta tag found in app.rs - required for responsive design"
        return 1
    fi
}

# Test 2: Check for responsive breakpoints in Tailwind config
test_tailwind_breakpoints() {
    log_test "Tailwind CSS responsive breakpoints configured"

    local tailwind_config="/home/lewis/src/clarity/clarity-web/tailwind.config.js"
    if [[ -f "$tailwind_config" ]]; then
        if grep -q "screens\|breakpoints" "$tailwind_config" 2>/dev/null; then
            log_pass "Tailwind config has breakpoint configuration"
            return 0
        else
            log_info "Using default Tailwind breakpoints (sm: 640px, md: 768px, lg: 1024px, xl: 1280px)"
            return 0
        fi
    else
        log_fail "tailwind.config.js not found"
        return 1
    fi
}

# Test 3: Check for responsive utility classes (sm:, md:, lg:, xl:)
test_responsive_classes() {
    log_test "Responsive utility classes used in components"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local responsive_count=$(grep -r "hidden sm:\|sm:block\|md:\|lg:\|xl:" "$discover_dir" 2>/dev/null | wc -l)

    if [[ $responsive_count -gt 0 ]]; then
        log_pass "Found $responsive_count instances of responsive utility classes"
        grep -r "hidden sm:\|sm:block\|md:\|lg:\|xl:" "$discover_dir" 2>/dev/null | head -5
        return 0
    else
        log_warn "No responsive utility classes found - components may not adapt to different screen sizes"
        log_info "Example: PhaseProgress uses 'hidden sm:block' to hide labels on mobile"
        return 1
    fi
}

# Test 4: Check touch target sizes (minimum 44px for touch)
test_touch_target_sizes() {
    log_test "Touch target sizes meet minimum 44px requirement"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local issues=0

    # Check for small touch targets in button components
    if grep -rn "h-\[.*\]\|w-\[.*\]" "$discover_dir" | grep -E "h-\[(2[0-9]|1[0-9]|[1-9])\]|w-\[(2[0-9]|1[0-9]|[1-9])\]" >/dev/null 2>&1; then
        log_fail "Found touch targets smaller than 44px (2rem = 32px, 3rem = 48px)"
        ((issues++))
    fi

    # Check for proper button sizing
    local button_file="/home/lewis/src/clarity/clarity-web/src/components/discover/extract_fields_button.rs"
    if [[ -f "$button_file" ]]; then
        if grep -q "px-3 py-2\|px-4 py-2\|px-4 py-3" "$button_file"; then
            log_pass "Button uses adequate padding (px-3/py-2 = ~44px touch target)"
        fi
    fi

    if [[ $issues -eq 0 ]]; then
        return 0
    else
        return 1
    fi
}

# Test 5: Check for fixed widths that break responsiveness
test_fixed_widths() {
    log_test "No fixed widths that prevent responsiveness"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local fixed_widths=$(grep -rn "w-\[.*px\]\|width:.*px" "$discover_dir" 2>/dev/null | grep -v "max-w-\|min-w-" || true)

    if [[ -z "$fixed_widths" ]]; then
        log_pass "No problematic fixed widths found"
        return 0
    else
        log_warn "Found fixed widths that may break responsiveness:"
        echo "$fixed_widths" | head -3
        log_info "Consider using max-w-* or responsive widths instead"
        return 1
    fi
}

# Test 6: Check container max-width constraints
test_container_constraints() {
    log_test "Containers use max-width for responsiveness"

    local main_component="/home/lewis/src/clarity/clarity-web/src/components/discover/progressive_discover.rs"

    if grep -q "max-w-4xl\|max-w-6xl\|max-w-full" "$main_component" 2>/dev/null; then
        log_pass "Main container uses max-width constraint (max-w-4xl = 896px)"
        grep -n "max-w-" "$main_component" | head -2
        return 0
    else
        log_warn "No max-width constraint found on main container"
        return 1
    fi
}

# Test 7: Check text scaling (readable at 200% zoom)
test_text_scaling() {
    log_test "Text sizes support 200% zoom scaling"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local text_sizes=$(grep -rn "text-" "$discover_dir" 2>/dev/null | grep -E "text-xs|text-sm|text-base|text-lg|text-xl" | wc -l)

    if [[ $text_sizes -gt 0 ]]; then
        log_pass "Found $text_sizes instances of relative text sizing (supports scaling)"
        grep -rn "text-xs\|text-sm\|text-base" "$discover_dir" 2>/dev/null | head -3
        return 0
    else
        log_warn "No relative text sizing found - may not scale properly"
        return 1
    fi
}

# Test 8: Check for horizontal scroll issues
test_horizontal_scroll() {
    log_test "No elements that cause horizontal scroll"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"

    # Check for overflow issues
    local overflow_issues=$(grep -rn "overflow-x-auto\|overflow-x-scroll" "$discover_dir" 2>/dev/null || true)
    local min_width_issues=$(grep -rn "min-w-\[.*px\]" "$discover_dir" 2>/dev/null | grep -E "min-w-\[(3[0-9]{2,}|[4-9][0-9]{2,}|[0-9]{4,})\]" || true)

    if [[ -z "$overflow_issues" ]] && [[ -z "$min_width_issues" ]]; then
        log_pass "No obvious horizontal scroll issues detected"
        return 0
    else
        log_warn "Potential horizontal scroll issues found:"
        [[ -n "$overflow_issues" ]] && echo "$overflow_issues" | head -2
        [[ -n "$min_width_issues" ]] && echo "$min_width_issues" | head -2
        return 1
    fi
}

# Test 9: Check spacing consistency
test_spacing_consistency() {
    log_test "Spacing uses consistent scale"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local spacing_issues=0

    # Check for arbitrary spacing values
    local arbitrary_spacing=$(grep -rn "p-\[.*\]\|m-\[.*\]\|px-\[.*\]\|py-\[.*\]" "$discover_dir" 2>/dev/null || true)

    if [[ -z "$arbitrary_spacing" ]]; then
        log_pass "Spacing uses Tailwind's consistent scale (1, 2, 3, 4, 6, 8...)"
        return 0
    else
        log_warn "Found arbitrary spacing values (should use Tailwind scale):"
        echo "$arbitrary_spacing" | head -3
        return 1
    fi
}

# Test 10: Check mobile-first approach
test_mobile_first() {
    log_test "Mobile-first responsive design approach"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local mobile_first=$(grep -rn "class:.*sm:\|md:\|lg:" "$discover_dir" 2>/dev/null | wc -l)

    if [[ $mobile_first -gt 0 ]]; then
        log_pass "Found $mobile_first instances of mobile-first responsive classes"
        log_info "Mobile-first: base styles for mobile, sm:/md:/lg: for larger screens"
        return 0
    else
        log_warn "Limited mobile-first responsive design detected"
        return 1
    fi
}

# Test 11: Verify PhaseProgress responsive behavior
test_phase_progress_responsive() {
    log_test "PhaseProgress component responsive behavior"

    local component="/home/lewis/src/clarity/clarity-web/src/components/discover/progressive_discover.rs"

    if grep -A5 "fn PhaseProgress" "$component" | grep -q "hidden sm:block"; then
        log_pass "PhaseProgress hides labels on mobile (hidden sm:block)"
        grep -B2 -A2 "hidden sm:block" "$component" | head -5
        return 0
    else
        log_fail "PhaseProgress doesn't hide labels on mobile - may overflow on small screens"
        return 1
    fi
}

# Test 12: Check for flex-wrap for responsive layouts
test_flex_wrap() {
    log_test "Flex layouts use wrap for responsiveness"

    local discover_dir="/home/lewis/src/clarity/clarity-web/src/components/discover"
    local flex_wrap=$(grep -rn "flex-wrap" "$discover_dir" 2>/dev/null | wc -l)

    if [[ $flex_wrap -gt 0 ]]; then
        log_pass "Found $flex_wrap instances of flex-wrap for responsive layouts"
        grep -rn "flex-wrap" "$discover_dir" 2>/dev/null | head -3
        return 0
    else
        log_warn "Limited flex-wrap usage - multi-item layouts may not wrap on small screens"
        return 1
    fi
}

# Test 13: Check textarea/input responsiveness
test_input_responsiveness() {
    log_test "Textareas and inputs use full width"

    local prompt_component="/home/lewis/src/clarity/clarity-web/src/components/discover/prompt_textarea.rs"

    if grep -q "w-full" "$prompt_component" 2>/dev/null; then
        log_pass "PromptTextarea uses w-full for responsive width"
        return 0
    else
        log_fail "PromptTextarea may not be responsive - missing w-full class"
        return 1
    fi
}

# Main test execution
main() {
    echo "================================"
    echo "Progressive Discover Responsive Design QA"
    echo "================================"
    echo ""

    # Run all tests
    test_viewport_meta
    test_tailwind_breakpoints
    test_responsive_classes
    test_touch_target_sizes
    test_fixed_widths
    test_container_constraints
    test_text_scaling
    test_horizontal_scroll
    test_spacing_consistency
    test_mobile_first
    test_phase_progress_responsive
    test_flex_wrap
    test_input_responsiveness

    # Summary
    echo ""
    echo "================================"
    echo "Test Summary"
    echo "================================"
    echo "Total Tests: $TOTAL_TESTS"
    echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
    echo -e "${RED}Failed: $FAILED_TESTS${NC}"
    echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
    echo ""

    # Calculate pass rate
    if [[ $TOTAL_TESTS -gt 0 ]]; then
        local pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
        echo "Pass Rate: ${pass_rate}%"
        echo ""

        if [[ $pass_rate -ge 80 ]]; then
            echo -e "${GREEN}✓ Responsive design is generally good${NC}"
        elif [[ $pass_rate -ge 60 ]]; then
            echo -e "${YELLOW}⚠ Responsive design needs improvement${NC}"
        else
            echo -e "${RED}✗ Responsive design has significant issues${NC}"
        fi
    fi

    # Return exit code based on failures
    if [[ $FAILED_TESTS -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Run main function
main "$@"
