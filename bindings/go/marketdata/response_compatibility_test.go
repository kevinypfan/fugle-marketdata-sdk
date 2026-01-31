// response_compatibility_test.go - Response structure validation tests
//
// These tests verify that response structs have expected fields,
// ensuring compatibility with the Fugle API response format.
//
// Run structural tests: CGO_ENABLED=1 go test -v -run Compatibility -short .
// Run integration tests: FUGLE_API_KEY=xxx CGO_ENABLED=1 go test -v -run Compatibility .

package marketdata_uniffi

import (
	"os"
	"reflect"
	"testing"
)

// ========== Quote Response Structure ==========

func TestQuoteCompatibility_HasSymbolField(t *testing.T) {
	quoteType := reflect.TypeOf(Quote{})

	field, found := quoteType.FieldByName("Symbol")
	if !found {
		t.Fatal("Quote struct should have Symbol field")
	}

	if field.Type.Kind() != reflect.String {
		t.Errorf("Quote.Symbol should be string, got %v", field.Type.Kind())
	}
}

func TestQuoteCompatibility_HasDateField(t *testing.T) {
	quoteType := reflect.TypeOf(Quote{})

	field, found := quoteType.FieldByName("Date")
	if !found {
		t.Fatal("Quote struct should have Date field")
	}

	if field.Type.Kind() != reflect.String {
		t.Errorf("Quote.Date should be string, got %v", field.Type.Kind())
	}
}

func TestQuoteCompatibility_HasExpectedFields(t *testing.T) {
	quoteType := reflect.TypeOf(Quote{})

	// Required fields
	requiredFields := []string{"Symbol", "Date"}
	for _, fieldName := range requiredFields {
		_, found := quoteType.FieldByName(fieldName)
		if !found {
			t.Errorf("Quote should have field: %s", fieldName)
		}
	}

	// Optional fields (should exist in struct)
	optionalFields := []string{"Name", "Exchange", "Market"}
	for _, fieldName := range optionalFields {
		_, found := quoteType.FieldByName(fieldName)
		if !found {
			t.Errorf("Quote should have field: %s", fieldName)
		}
	}
}

// ========== Ticker Response Structure ==========

func TestTickerCompatibility_HasSymbolField(t *testing.T) {
	tickerType := reflect.TypeOf(Ticker{})

	field, found := tickerType.FieldByName("Symbol")
	if !found {
		t.Fatal("Ticker struct should have Symbol field")
	}

	if field.Type.Kind() != reflect.String {
		t.Errorf("Ticker.Symbol should be string, got %v", field.Type.Kind())
	}
}

func TestTickerCompatibility_HasExpectedFields(t *testing.T) {
	tickerType := reflect.TypeOf(Ticker{})

	expectedFields := []string{"Symbol", "Date", "Name"}
	for _, fieldName := range expectedFields {
		_, found := tickerType.FieldByName(fieldName)
		if !found {
			t.Errorf("Ticker should have field: %s", fieldName)
		}
	}
}

// ========== Trades Response Structure ==========

func TestTradesResponseCompatibility_HasExpectedFields(t *testing.T) {
	tradesType := reflect.TypeOf(TradesResponse{})

	expectedFields := []string{"Symbol", "Date", "Data"}
	for _, fieldName := range expectedFields {
		_, found := tradesType.FieldByName(fieldName)
		if !found {
			t.Errorf("TradesResponse should have field: %s", fieldName)
		}
	}
}

func TestTradesResponseCompatibility_DataIsSlice(t *testing.T) {
	tradesType := reflect.TypeOf(TradesResponse{})

	field, found := tradesType.FieldByName("Data")
	if !found {
		t.Fatal("TradesResponse should have Data field")
	}

	if field.Type.Kind() != reflect.Slice {
		t.Errorf("TradesResponse.Data should be slice, got %v", field.Type.Kind())
	}
}

// ========== Integration Response Tests ==========

func TestQuoteResponseCompatibility_Integration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	apiKey := os.Getenv("FUGLE_API_KEY")
	if apiKey == "" {
		t.Skip("FUGLE_API_KEY not set")
	}

	client, err := NewRestClientWithApiKey(apiKey)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Destroy()

	quote, err := client.Stock().Intraday().GetQuote("2330")
	if err != nil {
		t.Fatalf("Failed to get quote: %v", err)
	}

	// Verify required fields are populated
	if quote.Symbol != "2330" {
		t.Errorf("Expected symbol 2330, got %s", quote.Symbol)
	}
	if quote.Date == "" {
		t.Error("Quote.Date should not be empty")
	}
}

func TestTickerResponseCompatibility_Integration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	apiKey := os.Getenv("FUGLE_API_KEY")
	if apiKey == "" {
		t.Skip("FUGLE_API_KEY not set")
	}

	client, err := NewRestClientWithApiKey(apiKey)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Destroy()

	ticker, err := client.Stock().Intraday().GetTicker("2330")
	if err != nil {
		t.Fatalf("Failed to get ticker: %v", err)
	}

	// Verify required fields are populated
	if ticker.Symbol != "2330" {
		t.Errorf("Expected symbol 2330, got %s", ticker.Symbol)
	}
	if ticker.Name == nil || *ticker.Name == "" {
		t.Error("Ticker.Name should not be empty")
	}
}

func TestTradesResponseCompatibility_Integration(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping integration test in short mode")
	}

	apiKey := os.Getenv("FUGLE_API_KEY")
	if apiKey == "" {
		t.Skip("FUGLE_API_KEY not set")
	}

	client, err := NewRestClientWithApiKey(apiKey)
	if err != nil {
		t.Fatalf("Failed to create client: %v", err)
	}
	defer client.Destroy()

	trades, err := client.Stock().Intraday().GetTrades("2330")
	if err != nil {
		t.Fatalf("Failed to get trades: %v", err)
	}

	// Verify required fields are populated
	if trades.Symbol != "2330" {
		t.Errorf("Expected symbol 2330, got %s", trades.Symbol)
	}
	if trades.Date == "" {
		t.Error("TradesResponse.Date should not be empty")
	}
	if trades.Data == nil {
		t.Error("TradesResponse.Data should not be nil")
	}
}
