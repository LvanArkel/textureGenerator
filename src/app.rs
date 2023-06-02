// ========= First, define your user data types =============

use super::Color;
use std::{collections::HashMap, borrow::Cow};

use eframe::egui::{self, DragValue};
use egui_node_graph::{NodeId, DataTypeTrait, NodeTemplateTrait, NodeTemplateIter, WidgetValueTrait, UserResponseTrait, NodeDataTrait, GraphEditorState, Graph, NodeResponse};
use image::{Rgb32FImage, Rgb};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
pub struct MyNodeData {
    template: MyNodeTemplate,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq)]
pub enum MyDataType {
    RgbColor,
    RgbImage,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency.
pub enum MyValueType {
    RgbColor { value: Color },
    RgbImage { value: Rgb32FImage },
}

impl Default for MyValueType {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::RgbColor { value: Rgb([0.0, 0.0, 0.0]) }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(EnumIter, Clone, Copy)]
pub enum MyNodeTemplate {
    SolidColor,
    BlendColor,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyGraphState {
    pub active_node: Option<NodeId>,
    pub cached_results: HashMap<NodeId, MyValueType>
}

// =========== Then, you need to implement some traits ============

impl DataTypeTrait<MyGraphState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> eframe::egui::Color32 {
        match self {
            MyDataType::RgbColor => egui::Color32::from_rgb(235, 76, 52),
            MyDataType::RgbImage => egui::Color32::from_rgb(52, 79, 235),
        }
    }

    fn name(&self) -> Cow<str> {
        match self {
            MyDataType::RgbColor => Cow::Borrowed("RGB Color"),
            MyDataType::RgbImage => Cow::Borrowed("RGB Image"),
        }
    }
}

impl NodeTemplateTrait for MyNodeTemplate {
    type NodeData = MyNodeData;
    type DataType = MyDataType;
    type ValueType = MyValueType;
    type UserState = MyGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        Cow::Borrowed(match self {
            MyNodeTemplate::SolidColor => "Solid color",
            MyNodeTemplate::BlendColor => "Blend color",
        })
    }

    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        match self {
            MyNodeTemplate::SolidColor => vec!["Generator nodes"],
            MyNodeTemplate::BlendColor => vec!["Transformation nodes"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        MyNodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut egui_node_graph::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            MyNodeTemplate::SolidColor => {
                graph.add_input_param(node_id, 
                    "color".into(), 
                    MyDataType::RgbColor,
                    MyValueType::RgbColor { value: Rgb([0.0, 0.0, 0.0]) }, 
                    egui_node_graph::InputParamKind::ConstantOnly, 
                    true);
                graph.add_output_param(node_id, "out".into(), MyDataType::RgbImage);
            },
            MyNodeTemplate::BlendColor => {
                graph.add_input_param(node_id,
                    "A".into(),
                    MyDataType::RgbImage, 
                    MyValueType::RgbImage { value: Rgb32FImage::new(0, 0) }, 
                    egui_node_graph::InputParamKind::ConnectionOnly, 
                    true);
                graph.add_input_param(node_id,
                    "B".into(),
                    MyDataType::RgbImage, 
                    MyValueType::RgbImage { value: Rgb32FImage::new(0, 0) }, 
                    egui_node_graph::InputParamKind::ConnectionOnly, 
                    true);
                graph.add_output_param(node_id, "out".into(), MyDataType::RgbImage);
            },
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MyNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        MyNodeTemplate::iter().collect()
    }
}

impl WidgetValueTrait for MyValueType {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type NodeData = MyNodeData;

    fn value_widget(
            &mut self,
            param_name: &str,
            node_id: NodeId,
            ui: &mut egui::Ui,
            user_state: &mut Self::UserState,
            node_data: &Self::NodeData,
        ) -> Vec<Self::Response> {
        match self {
            MyValueType::RgbColor { value } => {
                ui.label(param_name);
                ui.color_edit_button_rgb(&mut value.0);
            },
            MyValueType::RgbImage { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.label(value.width().to_string());
                    ui.label(value.height().to_string());
                });
            },
        }
        // This allows you to return your responses from the inline widgets.
        Vec::new()
    }
}

impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for MyNodeData {
    type Response = MyResponse;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        _graph: &egui_node_graph::Graph<MyNodeData, MyDataType, MyValueType>,
        user_state: &mut MyGraphState,
    ) -> Vec<egui_node_graph::NodeResponse<MyResponse, Self>>
    where
        MyResponse: UserResponseTrait,
    {
        user_state.cached_results.get(&node_id).map(|value| {
            match value {
                MyValueType::RgbColor { value } => {
                    ui.horizontal(|ui| {
                        ui.label("color:");
                        ui.label(value.0[0].to_string());
                        ui.label(value.0[1].to_string());
                        ui.label(value.0[2].to_string());
                    });
                },
                MyValueType::RgbImage { value } => {
                    ui.horizontal(|ui| {
                        ui.label("image:");
                        ui.label(format!("{}x{}", value.width(), value.height()));
                    });
                },
            }
        });
        Vec::new()
    }
}

type MyGraph = Graph<MyNodeData, MyDataType, MyValueType>;
type MyEditorState =
    GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

#[derive(Default)]
pub struct NodeGraphExample {
    state: MyEditorState,
    user_state: MyGraphState
}

impl eframe::App for NodeGraphExample {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let graph_response = egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.state.draw_graph_editor(
                    ui, 
                    AllMyNodeTemplates,
                    &mut self.user_state,
                    Vec::default(),
                )
            })
            .inner;
        for node_response in graph_response.node_responses {
            // Here, we ignore all other graph events. But you may find
            // some use for them. For example, by playing a sound when a new
            // connection is created
            if let NodeResponse::User(user_event) = node_response {
                match user_event {
                    MyResponse::SetActiveNode(node) => self.user_state.active_node = Some(node),
                    MyResponse::ClearActiveNode => self.user_state.active_node = None,
                }
            }
        }
    }
}

