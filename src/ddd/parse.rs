use anyhow::{anyhow, Result};

/// Parse name with optional index (format: YYYYMMDD[-N]-name)
/// Returns (date, index, name)
fn parse_name_with_index(parts: &[&str]) -> Result<(String, Option<usize>, String)> {
    // Must have at least date and name
    if parts.len() < 2 {
        return Err(anyhow!("Invalid name format: need at least date and name"));
    }

    // First part must be date (8 digits)
    let date = parts[0];
    validate_date_format(date)?;

    // Check if second part is an index
    if parts.len() >= 3 {
        if let Ok(index) = parts[1].parse::<usize>() {
            // Has index: date-index-name
            let name = parts[2..].join("-");
            validate_name_format(&name)?;
            return Ok((date.to_string(), Some(index), name));
        }
    }

    // No index: date-name
    let name = parts[1..].join("-");
    validate_name_format(&name)?;
    Ok((date.to_string(), None, name))
}

/// Parse feat directory name (format: YYYYMMDD[-N]-name)
pub fn parse_feat_name(name: &str) -> Result<(String, Option<usize>, String)> {
    let parts: Vec<&str> = name.split('-').collect();
    parse_name_with_index(&parts)
}

/// Parse single file name (format: YYYYMMDD[-N]-name.md)
pub fn parse_single_file_name(name: &str) -> Result<(String, Option<usize>, String)> {
    // Remove .md extension
    let name_without_ext = name
        .strip_suffix(".md")
        .ok_or_else(|| anyhow!("File must have .md extension"))?;

    let parts: Vec<&str> = name_without_ext.split('-').collect();
    parse_name_with_index(&parts)
}

/// Parse toy directory name (format: toyN_name, where name can have underscores)
pub fn parse_toy_name(name: &str) -> Result<(usize, String)> {
    // Must start with "toy"
    if !name.starts_with("toy") {
        return Err(anyhow!("Toy name must start with 'toy'"));
    }

    // Split on first underscore
    let parts: Vec<&str> = name.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid toy name format: {}", name));
    }

    // Parse number from "toyN" part
    let number_str = &parts[0][3..]; // Skip "toy" prefix
    let number = number_str
        .parse::<usize>()
        .map_err(|_| anyhow!("Invalid toy number: {}", number_str))?;

    let toy_name = parts[1].to_string();

    // Toy names can have underscores, so validate separately
    if toy_name.is_empty() {
        return Err(anyhow!("Toy name cannot be empty"));
    }
    if toy_name.starts_with('_') || toy_name.ends_with('_') {
        return Err(anyhow!(
            "Toy name cannot start or end with underscore: {}",
            toy_name
        ));
    }
    // Check that it only contains lowercase, digits, and underscores
    for ch in toy_name.chars() {
        if !ch.is_lowercase() && !ch.is_ascii_digit() && ch != '_' {
            return Err(anyhow!(
                "Toy name must be lowercase with underscores, got: {}",
                toy_name
            ));
        }
    }

    Ok((number, toy_name))
}

/// Validate date format (YYYYMMDD)
pub fn validate_date_format(date: &str) -> Result<()> {
    if date.len() != 8 {
        return Err(anyhow!("Date must be 8 digits (YYYYMMDD), got: {}", date));
    }

    if !date.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("Date must contain only digits, got: {}", date));
    }

    Ok(())
}

/// Validate name format (lowercase with hyphens)
pub fn validate_name_format(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Name cannot be empty"));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow!("Name cannot start or end with hyphen: {}", name));
    }

    for ch in name.chars() {
        if !ch.is_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            return Err(anyhow!(
                "Name must be lowercase with hyphens, got: {}",
                name
            ));
        }
    }

    Ok(())
}
