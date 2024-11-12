use biome_analyze::{
    AddVisitor, FromServices, MissingServicesDiagnostic, Phase, Phases, QueryMatch, Queryable,
    RuleKey, ServiceBag, Visitor, VisitorContext, VisitorFinishContext,
};
use biome_js_dependency_graph::DependencyGraphModel;
use biome_js_syntax::{AnyJsRoot, JsLanguage, JsSyntaxNode, TextRange, WalkEvent};

pub struct DependencyGraph {
    pub model: DependencyGraphModel,
}

impl FromServices for DependencyGraph {
    fn from_services(
        rule_key: &RuleKey,
        services: &ServiceBag,
    ) -> Result<Self, MissingServicesDiagnostic> {
        let model: &DependencyGraphModel = services.get_service().ok_or_else(|| {
            MissingServicesDiagnostic::new(rule_key.rule_name(), &["DependencyGraphModel"])
        })?;
        Ok(Self {
            model: model.clone(),
        })
    }
}

impl Phase for DependencyGraph {
    fn phase() -> Phases {
        Phases::DependencyGraph
    }
}

/// The [DependencyGraphServices] types can be used as a queryable to get an
/// instance of the whole [DependencyGraphModel] without matching on a specific
/// AST node.
impl Queryable for DependencyGraph {
    type Input = DependencyGraphModelEvent;
    type Output = DependencyGraphModel;

    type Language = JsLanguage;
    type Services = Self;

    fn build_visitor(analyzer: &mut impl AddVisitor<JsLanguage>, root: &AnyJsRoot) {
        analyzer.add_visitor(Phases::Syntax, || {
            DependencyGraphModelBuilderVisitor::new(root)
        });
        analyzer.add_visitor(Phases::DependencyGraph, || DependencyGraphModelVisitor);
    }

    fn unwrap_match(services: &ServiceBag, _: &DependencyGraphModelEvent) -> Self::Output {
        services
            .get_service::<DependencyGraphModel>()
            .expect("DependencyGraph service is not registered")
            .clone()
    }
}

pub struct DependencyGraphModelBuilderVisitor {
    // TODO
}

impl DependencyGraphModelBuilderVisitor {
    pub(crate) fn new(root: &AnyJsRoot) -> Self {
        Self {}
    }
}

impl Visitor for DependencyGraphModelBuilderVisitor {
    type Language = JsLanguage;

    fn visit(&mut self, event: &WalkEvent<JsSyntaxNode>, _ctx: VisitorContext<JsLanguage>) {
        // TODO
    }

    fn finish(self: Box<Self>, ctx: VisitorFinishContext<JsLanguage>) {
        let model = DependencyGraphModel::new();
        ctx.services.insert_service(model);
    }
}

pub struct DependencyGraphModelVisitor;

pub struct DependencyGraphModelEvent(TextRange);

impl QueryMatch for DependencyGraphModelEvent {
    fn text_range(&self) -> TextRange {
        self.0
    }
}

impl Visitor for DependencyGraphModelVisitor {
    type Language = JsLanguage;

    fn visit(&mut self, event: &WalkEvent<JsSyntaxNode>, mut ctx: VisitorContext<JsLanguage>) {
        let root = match event {
            WalkEvent::Enter(node) => {
                if node.parent().is_some() {
                    return;
                }

                node
            }
            WalkEvent::Leave(_) => return,
        };

        let text_range = root.text_range_with_trivia();
        ctx.match_query(DependencyGraphModelEvent(text_range));
    }
}
