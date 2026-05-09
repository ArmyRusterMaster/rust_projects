# data_prep.md

Guidelines for preparing data for ML projects:
- Data collection, cleaning, normalization
- Handling missing values
- Feature engineering basics
- Data splitting: train/validation/test
- Data versioning and provenance

Example snippet:

```python
import pandas as pd
from sklearn.model_selection import train_test_split

df = pd.read_csv('data.csv')
X = df.drop('target', axis=1)
y = df['target']
X_train, X_val, y_train, y_val = train_test_split(X, y, test_size=0.2, random_state=42)
```
