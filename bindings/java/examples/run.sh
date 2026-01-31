#!/bin/bash
# Java 範例執行腳本
#
# 用法:
#   ./run.sh rest       # 執行 REST 範例
#   ./run.sh websocket  # 執行 WebSocket 範例
#
# 需要先設定:
#   export FUGLE_API_KEY='your-api-key'
#   export JAVA_HOME=/path/to/java21  (如果不在 PATH)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
JAVA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$JAVA_DIR/../.." && pwd)"

# 檢查 API Key
if [ -z "$FUGLE_API_KEY" ]; then
    echo "錯誤: 請先設定 FUGLE_API_KEY 環境變數"
    echo "  export FUGLE_API_KEY='your-api-key'"
    exit 1
fi

# 檢查 native library
NATIVE_LIB="$PROJECT_ROOT/target/release/libmarketdata_uniffi.dylib"
if [ ! -f "$NATIVE_LIB" ]; then
    echo "Native library 不存在，先 build..."
    cd "$PROJECT_ROOT"
    cargo build -p marketdata-uniffi --release
fi

# 編譯 Java
echo "編譯 Java..."
cd "$JAVA_DIR"
./gradlew compileJava -q

# 找到 JNA jar
JNA_JAR=$(find ~/.gradle -name 'jna-5*.jar' 2>/dev/null | head -1)
if [ -z "$JNA_JAR" ]; then
    echo "找不到 JNA jar，執行 gradle build 下載..."
    ./gradlew build -x test -q
    JNA_JAR=$(find ~/.gradle -name 'jna-5*.jar' 2>/dev/null | head -1)
fi

if [ -z "$JNA_JAR" ]; then
    echo "錯誤: 找不到 JNA jar"
    exit 1
fi

# 設定 classpath
CLASSPATH="$JAVA_DIR/build/classes/java/main:$JNA_JAR"

# 設定 native library path
export LD_LIBRARY_PATH="$PROJECT_ROOT/target/release:$LD_LIBRARY_PATH"
export DYLD_LIBRARY_PATH="$PROJECT_ROOT/target/release:$DYLD_LIBRARY_PATH"

# 執行範例
case "${1:-rest}" in
    rest)
        echo ""
        echo "=== 執行 REST 範例 ==="
        echo ""
        java -cp "$CLASSPATH" \
             -Djna.library.path="$PROJECT_ROOT/target/release" \
             tw.com.fugle.marketdata.examples.RestExample
        ;;
    websocket|ws)
        echo ""
        echo "=== 執行 WebSocket 範例 ==="
        echo ""
        java -cp "$CLASSPATH" \
             -Djna.library.path="$PROJECT_ROOT/target/release" \
             tw.com.fugle.marketdata.examples.WebSocketExample
        ;;
    *)
        echo "用法: $0 [rest|websocket]"
        echo ""
        echo "  rest      - 執行 REST API 範例"
        echo "  websocket - 執行 WebSocket 串流範例"
        exit 1
        ;;
esac
