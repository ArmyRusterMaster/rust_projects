# usage_patterns.md

Common ML workflows and patterns:
- Data prep: cleaning, normalization, feature engineering
- Model selection: baseline vs complex models; cross-validation
- Training loops: batch sizing, learning rate schedule, early stopping
- Evaluation: metrics per task, conf/roc curves, cross-validation scores
- Reproducibility: seeds, deterministic ops, frozen envs, model versioning
- Deployment: saving models, export formats, serving

Examples:
```
# Simple pipeline with scikit-learn
from sklearn.pipeline import Pipeline
from sklearn.preprocessing import StandardScaler
from sklearn.linear_model import LogisticRegression

pipe = Pipeline([
  ('scaler', StandardScaler()),
  ('clf', LogisticRegression(max_iter=1000))
])
pipe.fit(X_train, y_train)
pred = pipe.predict(X_test)
```
