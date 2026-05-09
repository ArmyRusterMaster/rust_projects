use tract_onnx::prelude::*;

pub struct MnistModel {
    model: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
}

impl MnistModel {
    pub fn new() -> anyhow::Result<Self> {
        let model_data = std::fs::read("models/mnist-8.onnx")?;
        let model = tract_onnx::onnx()
            .model_for_read(&mut model_data.as_slice())?
            .into_optimized()?
            .into_runnable()?;
        Ok(MnistModel { model })
    }

    pub fn predict(&self, pixels: &[f32]) -> Option<u8> {
        // pixels: 784 элемента f32
        if pixels.len() != 784 {
            return None;
        }
        // Преобразуем в тензор shape (1, 1, 28, 28)
        let input = Tensor::from_shape(&[1, 1, 28, 28], pixels).ok()?;
        let result = self.model.run(tvec!(input)).ok()?;
        let output = result[0].to_array_view::<f32>().ok()?;
        // output shape (1, 10)
        let probs = output.iter().enumerate();
        probs.max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(idx, _)| idx as u8)
    }
}
