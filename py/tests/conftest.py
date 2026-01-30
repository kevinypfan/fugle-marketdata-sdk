"""Pytest configuration for marketdata-py tests."""
import pytest


@pytest.fixture(scope="session")
def mock_api_key():
    """Provide a mock API key for testing."""
    return "mock_api_key_for_testing_purposes"
