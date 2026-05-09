# ml_libs.md

Цель: обзор ключевых ML-библиотек, установка, базовые рабочие паттерны, примеры.

Установка и общие заметки
- Рекомендуется создавать виртуальное окружение (venv или conda).
- Используйте совместимые версии Python (обычно 3.8–3.11).
- Для GPU-ускорения устанавливайте версии библиотек, совместимые с вашей видеокартой и CUDA/cuDNN.
- Для воспроизводимости фиксируйте версии библиотек (requirements.txt или poetry.lock).

Скит-лерн (scikit-learn)
- Установка: pip install scikit-learn
- Основной API: классификация/регрессия/кластинг; пайплайны; трансформеры.
- Пример:

```python
from sklearn.datasets import load_iris
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score

X, y = load_iris(return_X_y=True)
X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)
clf = RandomForestClassifier(n_estimators=100, random_state=42)
clf.fit(X_train, y_train)
pred = clf.predict(X_test)
acc = accuracy_score(y_test, pred)
print(acc)
```

TensorFlow (Keras встроенная)
- Установка: pip install tensorflow
- Примечание: в TensorFlow 2.x Keras интегрирована как tf.keras.
- Пример:

```python
import tensorflow as tf
from tensorflow.keras import layers, models

model = models.Sequential([
    layers.Dense(64, activation='relu', input_shape=(4,)),
    layers.Dense(3, activation='softmax')
])
model.compile(optimizer='adam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])

import numpy as np
X = np.random.random((100, 4))
y = np.random.randint(0, 3, size=(100,))
model.fit(X, y, epochs=5)
```

PyTorch
- Установка: pip install torch torchvision
- Пример:

```python
import torch
import torch.nn as nn
import torch.optim as optim

X = torch.randn(100, 4)
y = torch.randint(0, 3, (100,))
model = nn.Sequential(nn.Linear(4, 64), nn.ReLU(), nn.Linear(64, 3))
criterion = nn.CrossEntropyLoss()
optimizer = optim.Adam(model.parameters())

for epoch in range(5):
    optimizer.zero_grad()
    out = model(X)
    loss = criterion(out, y)
    loss.backward()
    optimizer.step()
```

JAX
- Установка: pip install jax jaxlib
- Пример:

```python
import jax
import jax.numpy as jnp

@jax.jit
def f(x):
    return jnp.sin(x)

print(f(2.0))
```

Дополнительные заметки
- Сравнение и выбор: используйте scikit-learn для быстрых прототипов и структурированных данных; TensorFlow/PyTorch — для глубокого обучения; JAX — для экспериментальных и высокопроизводительных задач.
- Воспроизводимость: фиксируйте seed, используйте фиксированные данные, сохраняйте модель и конфигурацию.
- Варианты развёртывания: ONNX как мост между фреймворками; экспорт моделей в common formats.
