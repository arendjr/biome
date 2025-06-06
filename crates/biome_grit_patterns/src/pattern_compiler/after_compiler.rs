use super::{PatternCompiler, compilation_context::NodeCompilationContext};
use crate::{CompileError, grit_context::GritQueryContext};
use biome_grit_syntax::GritPatternAfter;
use grit_pattern_matcher::pattern::After;

pub(crate) struct AfterCompiler;

impl AfterCompiler {
    pub(crate) fn from_node(
        node: &GritPatternAfter,
        context: &mut NodeCompilationContext,
    ) -> Result<After<GritQueryContext>, CompileError> {
        let pattern = PatternCompiler::from_node(&node.pattern()?, context)?;
        Ok(After::new(pattern))
    }
}
