# GPT-OSS-Safeguard Policy Templates

This directory contains policy templates for the GPT-OSS-Safeguard provider.

## Files

### `mlcommons_taxonomy_policy.txt`

**Purpose**: MLCommons safety taxonomy (S1-S13) that mirrors Llama Guard 3 categories.

**Usage**:
- Default policy for `GptOssSafeguardConfig::default()`
- Makes GPT-OSS-Safeguard behave identically to Llama Guard 3
- Loaded at compile time via `include_str!()` macro

**Categories covered**: All 13 MLCommons categories
- S1: Violent Crimes
- S2: Non-Violent Crimes
- S3: Sex-Related Crimes
- S4: Child Sexual Exploitation
- S5: Defamation
- S6: Specialized Advice
- S7: Privacy Violations
- S8: Intellectual Property
- S9: Indiscriminate Weapons
- S10: Hate Speech
- S11: Suicide & Self-Harm
- S12: Sexual Content
- S13: Elections

**Format**: Policy includes:
- Policy definitions for each category
- VIOLATES section with violation criteria
- DOES NOT Violate section with safe content examples
- Output format specification (JSON)
- 10 example scenarios

## Adding Custom Policies

To add a new policy template:

1. Create a new `.txt` file in this directory
2. Follow the structure of `mlcommons_taxonomy_policy.txt`:
   - Policy name/header
   - Definitions section
   - VIOLATES Policy (Label: 1) section
   - DOES NOT Violate Policy (Label: 0) section
   - Output format specification
   - Examples (4-10 scenarios)

3. Load in code using `include_str!()`:
   ```rust
   const CUSTOM_POLICY: &str = include_str!("policies/my_custom_policy.txt");
   ```

## Editing Policies

**Important**: Policies are embedded at **compile time**. After editing:

```bash
# Rebuild to pick up changes
cargo build --release
```

**Benefits of external policy files**:
- ✅ Easier to edit (no Rust syntax required)
- ✅ Version control for policy changes
- ✅ Can be reviewed by non-developers
- ✅ Zero runtime overhead (compiled into binary)
- ✅ Consistent with pattern file approach

## Policy Structure Requirements

All GPT-OSS-Safeguard policies must include:

1. **Policy Definitions** - Define key terms and categories
2. **VIOLATES section** - Clear criteria for violations
3. **DOES NOT Violate section** - Examples of safe content
4. **Output Format** - Specify JSON format expected
5. **Examples** - At least 4 examples (2 violations, 2 safe)

Without these sections, the model may produce inconsistent or incorrect classifications.
