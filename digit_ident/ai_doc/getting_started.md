# getting_started.md

Quickstart for ML projects:
- Create virtual environment
- Install core libs: scikit-learn, numpy, pandas
- Pick a task (classification/regression) and a baseline model
- Split data into train/val/test
- Evaluate and iterate

Example snippet:

```python
from sklearn.datasets import load_boston
from sklearn.model_selection import train_test_split
from sklearn.ensemble import RandomForestRegressor
from sklearn.metrics import mean_squared_error

X, y = load_boston(return_X_y=True)
X_train, X_val, y_train, y_val = train_test_split(X, y, test_size=0.2, random_state=42)
model = RandomForestRegressor(n_estimators=100, random_state=42)
model.fit(X_train, y_train)
pred = model.predict(X_val)
rmse = mean_squared_error(y_val, pred, squared=False)
print(rmse)
```

Notes:
- Use virtualenv/conda; pin versions
- Seed for reproducibility
- Track experiments with simple notebook or script
