use rand::distr::Distribution;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)] // Add PartialEq, Eq, Hash for potential Set usage if needed, but not strictly for this solution
pub struct Authority {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub public_key: [u8; 32],
    pub priority: u8, // 0..100 (lower = higher)
}

impl Authority {
    pub fn new(name: String, host: String, port: u16, public_key: [u8; 32], priority: u8) -> Self {
        Self {
            name,
            host,
            port,
            public_key,
            priority,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}@{}:{}", self.name, self.host, self.port)
    }

    pub fn to_vec(&self) -> Vec<Authority> {
        vec![self.clone()]
    }
}

// Existing pick_weighted function (corrected from previous response)
pub fn pick_weighted(list: &[Authority]) -> Option<&Authority> {
    use rand::distr::weighted::WeightedIndex;
    use rand::rng;

    if list.is_empty() {
        return None;
    }

    let mut rng = rng();

    // Calculate weights: lower priority (e.g., 0) should have higher weight (101)
    // higher priority (e.g., 100) should have lower weight (1)
    // Priority is 0..100, so 101 - priority will be 1..101, always positive.
    let weights: Vec<u32> = list.iter().map(|a| 101 - a.priority as u32).collect();

    let dist = WeightedIndex::new(&weights).ok()?;

    let index = dist.sample(&mut rng);

    Some(&list[index])
}

/// Picks K unique Authority sources from the list, weighted by their priority.
/// Lower priority values (e.g., 0) result in higher chances of being picked.
///
/// Args:
///   list: The slice of Authority instances to pick from.
///   k: The number of unique sources to pick.
///
/// Returns:
///   A vector containing up to K unique references to Authority instances.
///   If k is greater than the number of available unique sources, it returns all available sources.
pub fn pick_k_weighted_unique<'a>(list: &'a [Authority], k: usize) -> Vec<&'a Authority> {
    use rand::distr::weighted::WeightedIndex;
    use rand::rng;

    let mut result: Vec<&'a Authority> = Vec::with_capacity(k);

    if list.is_empty() || k == 0 {
        return result;
    }

    // We use a vector of indices to represent the currently available sources.
    // This allows us to efficiently remove sources as they are picked.
    let mut available_indices: Vec<usize> = (0..list.len()).collect();

    let mut rng = rng();

    for _ in 0..k {
        // If there are no more unique sources left to pick, break the loop.
        if available_indices.is_empty() {
            break;
        }

        // Create a temporary list of references to the *currently available* RootAuthorities
        // and calculate their weights.
        let current_available_authorities: Vec<&Authority> = available_indices
            .iter()
            .map(|&original_idx| &list[original_idx])
            .collect();

        let weights: Vec<u32> = current_available_authorities
            .iter()
            .map(|a| 101 - a.priority as u32)
            .collect();

        // It's safe to unwrap WeightedIndex::new here because weights will always be positive (1 to 101)
        // given priority is 0..100 and available_indices is not empty.
        let dist = WeightedIndex::new(&weights)
            .expect("Failed to create WeightedIndex with positive weights");

        // Sample an index from the `current_available_authorities` list.
        // This `sampled_relative_index` refers to the position within `current_available_authorities`
        // and also within `available_indices`.
        let sampled_relative_index = dist.sample(&mut rng);

        // Get the picked Authority and add it to our result.
        let picked_authority = current_available_authorities[sampled_relative_index];
        result.push(picked_authority);

        // Remove the selected item's original index from `available_indices`
        // to ensure it's not picked again.
        available_indices.remove(sampled_relative_index);
    }

    result
}
pub fn byzantine_quorum_k(n: usize) -> usize {
    if n < 4 { 1 } else { (2 * n + 2) / 3 }
}
