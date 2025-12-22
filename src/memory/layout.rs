//! Memory layout optimization
//! Structure packing and cache-friendly layouts
use anyhow::Result;
/// Memory alignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryAlignment {
    Align1,
    Align2,
    Align4,
    Align8,
    Align16,
    Align32,
    Align64,
}
/// Structure packing strategy
#[derive(Debug, Clone)]
pub struct StructureLayout {
    pub size: usize,
    pub alignment: usize,
    pub fields: Vec<FieldLayout>,
}
/// Field layout info
#[derive(Debug, Clone)]
pub struct FieldLayout {
    pub offset: usize,
    pub size: usize,
    pub alignment: usize,
}
/// Memory layout optimizer
pub struct MemoryLayoutOptimizer {
    cache_line_size: usize,
}
impl MemoryLayoutOptimizer {
    pub fn new() -> Self {
        Self {
            cache_line_size: 64, // Typical cache line size
        }
    }
    /// Optimize structure layout
    pub fn optimize_structure(&self, fields: &[FieldLayout]) -> Result<StructureLayout> {
        let mut optimized = fields.to_vec();
        optimized.sort_by_key(|f| f.alignment);
        let mut layout = StructureLayout {
            size: 0,
            alignment: 1,
            fields: Vec::new(),
        };
        for field in optimized {
            let padding: _ = self.calculate_padding(layout.size, field.alignment);
            let offset: _ = layout.size + padding;
            layout.size = offset + field.size;
            layout.alignment = layout.alignment.max(field.alignment);
            layout.fields.push(FieldLayout {
                offset,
                size: field.size,
                alignment: field.alignment,
            });
        }
        Ok(layout)
    }
    /// Calculate padding
    fn calculate_padding(&self, current_offset: usize, required_alignment: usize) -> usize {
        let misalignment: _ = current_offset % required_alignment;
        if misalignment == 0 {
            0
        } else {
            required_alignment - misalignment
        }
    }
    /// Optimize for cache lines
    pub fn optimize_cache_lines(&self, layout: &mut StructureLayout) {
        let cache_aligned_fields: Vec<_> = layout.fields
            .chunks(self.cache_line_size / std::mem::size_of::<usize>())
            .map(|chunk| chunk.to_vec())
            .collect();
        layout.fields = cache_aligned_fields.into_iter().flatten().collect();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
use std::collections::<HashMap, BTreeMap>;
    #[test]
    fn test_memory_layout_optimizer() {
        let optimizer: _ = MemoryLayoutOptimizer::new();
        assert_eq!(optimizer.cache_line_size, 64);
    }
    #[test]
    fn test_optimize_structure() {
        let optimizer: _ = MemoryLayoutOptimizer::new();
        let fields: _ = vec![
            FieldLayout { offset: 0, size: 8, alignment: 8 },
            FieldLayout { offset: 0, size: 4, alignment: 4 },
            FieldLayout { offset: 0, size: 1, alignment: 1 },
        ];
        let layout: _ = optimizer.optimize_structure(&fields).unwrap();
        assert!(layout.size > 0);
        assert!(layout.alignment > 0);
    }
}