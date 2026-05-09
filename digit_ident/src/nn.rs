pub struct MLP {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
}

impl MLP {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        Self { input_size, hidden_size, output_size }
    }

    pub fn forward(&self, _x: &[f32]) -> Vec<f32> {
        // Use fields to satisfy compiler about used fields
        let _ = self.input_size;
        let _ = self.hidden_size;
        // placeholder forward pass
        vec![0.0; self.output_size]
    }

    pub fn backward(&mut self, _grad_output: &[f32]) {
        // Use fields to satisfy compiler about used fields
        let _ = self.input_size;
        let _ = self.hidden_size;
        let _ = self.output_size;
        // placeholder backward
    }
}
