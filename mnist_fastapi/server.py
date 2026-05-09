import io
import numpy as np
from fastapi import FastAPI, File, UploadFile
from PIL import Image
import onnxruntime as ort

app = FastAPI(title="MNIST Digit Recognizer")

session = ort.InferenceSession("mnist.onnx")

def preprocess(image_bytes):
    img = Image.open(io.BytesIO(image_bytes)).convert("L")
    img = img.resize((28, 28), Image.Resampling.LANCZOS)
    img = np.array(img, dtype=np.float32) / 255.0
    img = 1.0 - img  # инверсия
    img = img.reshape(1, 1, 28, 28)
    return img

@app.post("/predict")
async def predict(file: UploadFile = File(...)):
    contents = await file.read()
    input_tensor = preprocess(contents)
    outputs = session.run(["Output"], {"Input3": input_tensor})
    probs = outputs[0][0]
    digit = int(np.argmax(probs))
    confidence = float(np.max(probs))
    return {"digit": digit, "confidence": confidence}

@app.get("/health")
async def health():
    return {"status": "ok"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
