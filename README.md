# geo-contains

GeoJSONポリゴン内に緯度経度が含まれるかを判定するRustツール

## 概要

このツールは、大きなGeoJSONポリゴンデータ（1MB〜10MB）を効率的に処理し、与えられた緯度経度がポリゴン内に含まれるかどうかを判定します。空間インデックス（Uber H3）を使用して、処理を高速化することもできます。

## 機能

- 単一のGeoJSONファイルに対する点の包含判定
- 複数のGeoJSONファイルに対する点の包含判定
- 空間インデックス（Uber H3）を使用した高速な判定
- 並列処理による複数ファイルの効率的な処理

## 使用方法

### 単一ファイルの判定

```bash
cargo run -- check --lat 35.6812 --lng 139.7671 --file data/tokyo.geojson
```

空間インデックスを使用する場合：

```bash
cargo run -- check --lat 35.6812 --lng 139.7671 --file data/tokyo.geojson --use-index
```

### 複数ファイルの判定

```bash
cargo run -- batch-check --lat 35.6812 --lng 139.7671 --dir data/
```

空間インデックスを使用する場合：

```bash
cargo run -- batch-check --lat 35.6812 --lng 139.7671 --dir data/ --use-index
```

### オプション

- `--resolution` または `-r`: H3解像度（0-15、デフォルト: 9）

## 依存クレート

- geo: 地理空間データの処理
- geojson: GeoJSONファイルの読み込みと解析
- h3o: Uber H3による空間インデックス
- serde, serde_json: JSONデータのシリアライズ/デシリアライズ
- clap: コマンドライン引数の解析
- rayon: 並列処理
- thiserror: エラー処理

## ライセンス

MIT
