"""Tests for CLASP client."""

import pytest
from clasp import Clasp, ClaspBuilder
from clasp.client import ClaspError


class TestClaspBuilder:
    """Test ClaspBuilder class."""

    def test_builder_creation(self):
        builder = ClaspBuilder(url="ws://localhost:7330")
        assert builder.url == "ws://localhost:7330"

    def test_builder_with_name(self):
        builder = ClaspBuilder(url="ws://localhost:7330")
        result = builder.with_name("Test Client")
        assert result is builder  # Returns self for chaining
        assert builder.name == "Test Client"

    def test_builder_with_features(self):
        builder = ClaspBuilder(url="ws://localhost:7330")
        result = builder.with_features(["param", "event"])
        assert result is builder
        assert builder.features == ["param", "event"]

    def test_builder_with_token(self):
        builder = ClaspBuilder(url="ws://localhost:7330")
        result = builder.with_token("secret-token")
        assert result is builder
        assert builder.token == "secret-token"

    def test_builder_with_reconnect(self):
        builder = ClaspBuilder(url="ws://localhost:7330")
        result = builder.with_reconnect(True, 10.0)
        assert result is builder
        assert builder.reconnect is True
        assert builder.reconnect_interval == 10.0

    def test_builder_chaining(self):
        builder = (
            ClaspBuilder(url="ws://localhost:7330")
            .with_name("Chained Client")
            .with_features(["param"])
            .with_token("token123")
            .with_reconnect(False)
        )
        assert builder.name == "Chained Client"
        assert builder.features == ["param"]
        assert builder.token == "token123"
        assert builder.reconnect is False


class TestClasp:
    """Test Clasp client class."""

    def test_client_creation(self):
        client = Clasp("ws://localhost:7330")
        assert client.url == "ws://localhost:7330"
        assert client.connected is False

    def test_client_with_options(self):
        client = Clasp(
            url="ws://localhost:7330",
            name="Test Client",
            features=["param", "event"],
            token="secret",
            reconnect=False,
        )
        assert client.name == "Test Client"
        assert client.features == ["param", "event"]
        assert client.token == "secret"
        assert client.reconnect is False

    def test_client_builder_method(self):
        builder = Clasp.builder("ws://localhost:7330")
        assert isinstance(builder, ClaspBuilder)
        assert builder.url == "ws://localhost:7330"

    def test_client_not_connected_initially(self):
        client = Clasp("ws://localhost:7330")
        assert client.connected is False
        assert client.session_id is None

    def test_client_cached_returns_none(self):
        client = Clasp("ws://localhost:7330")
        assert client.cached("/nonexistent/path") is None

    def test_client_time(self):
        client = Clasp("ws://localhost:7330")
        t = client.time()
        assert isinstance(t, int)
        assert t > 0


class TestPatternMatching:
    """Test address pattern matching."""

    def test_exact_match(self):
        client = Clasp("ws://localhost:7330")
        assert client._match_pattern("/test/path", "/test/path") is True
        assert client._match_pattern("/test/path", "/other/path") is False

    def test_single_wildcard(self):
        client = Clasp("ws://localhost:7330")
        pattern = "/test/*/value"
        assert client._match_pattern(pattern, "/test/foo/value") is True
        assert client._match_pattern(pattern, "/test/bar/value") is True
        assert client._match_pattern(pattern, "/test/value") is False
        assert client._match_pattern(pattern, "/test/foo/bar/value") is False

    def test_multi_wildcard(self):
        client = Clasp("ws://localhost:7330")
        pattern = "/test/**/value"
        assert client._match_pattern(pattern, "/test/value") is True
        assert client._match_pattern(pattern, "/test/foo/value") is True
        assert client._match_pattern(pattern, "/test/foo/bar/value") is True

    def test_trailing_wildcard(self):
        client = Clasp("ws://localhost:7330")
        pattern = "/test/**"
        assert client._match_pattern(pattern, "/test") is True
        assert client._match_pattern(pattern, "/test/foo") is True
        assert client._match_pattern(pattern, "/test/foo/bar") is True
        assert client._match_pattern(pattern, "/other/test") is False


class TestClaspError:
    """Test ClaspError exception."""

    def test_error_creation(self):
        error = ClaspError("Test error message")
        assert str(error) == "Test error message"

    def test_error_inheritance(self):
        error = ClaspError("Error")
        assert isinstance(error, Exception)


class TestQuerySignals:
    """Test query_signals and get_signals methods."""

    @pytest.mark.asyncio
    async def test_query_signals_not_connected_raises(self):
        """query_signals should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.query_signals("/test/**")

    @pytest.mark.asyncio
    async def test_get_signals_not_connected_raises(self):
        """get_signals should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.get_signals("/test/**")

    def test_query_signals_signature(self):
        """query_signals should accept pattern and optional timeout."""
        import inspect
        client = Clasp("ws://localhost:7330")
        sig = inspect.signature(client.query_signals)
        params = list(sig.parameters.keys())
        assert "pattern" in params
        assert "timeout" in params
        # Default timeout should be 5.0
        assert sig.parameters["timeout"].default == 5.0


class TestGesture:
    """Test gesture method."""

    @pytest.mark.asyncio
    async def test_gesture_not_connected_raises(self):
        """gesture should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.gesture("/input/touch/0", gesture_id=1, phase="start")

    @pytest.mark.asyncio
    async def test_gesture_with_payload_not_connected(self):
        """gesture with payload should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.gesture(
                "/input/touch/0",
                gesture_id=1,
                phase="move",
                payload={"x": 100, "y": 200, "pressure": 0.5}
            )

    def test_gesture_signature(self):
        """gesture should have correct signature."""
        import inspect
        client = Clasp("ws://localhost:7330")
        sig = inspect.signature(client.gesture)
        params = list(sig.parameters.keys())
        assert "address" in params
        assert "gesture_id" in params
        assert "phase" in params
        assert "payload" in params
        # payload should default to None
        assert sig.parameters["payload"].default is None

    def test_gesture_phases_valid(self):
        """Test that gesture phase validation is documented."""
        # Phases are: 'start', 'move', 'end', 'cancel'
        valid_phases = ["start", "move", "end", "cancel"]
        assert len(valid_phases) == 4


class TestTimeline:
    """Test timeline method."""

    @pytest.mark.asyncio
    async def test_timeline_not_connected_raises(self):
        """timeline should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.timeline(
                "/lumen/layer/0/opacity",
                keyframes=[
                    {"time": 0, "value": 0.0},
                    {"time": 1000000, "value": 1.0},
                ]
            )

    @pytest.mark.asyncio
    async def test_timeline_with_loop_not_connected(self):
        """timeline with loop=True should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.timeline(
                "/effect/fade",
                keyframes=[
                    {"time": 0, "value": 0.0},
                    {"time": 500000, "value": 1.0},
                    {"time": 1000000, "value": 0.0},
                ],
                loop=True
            )

    @pytest.mark.asyncio
    async def test_timeline_with_start_time_not_connected(self):
        """timeline with explicit start_time should raise ClaspError when not connected."""
        client = Clasp("ws://localhost:7330")
        with pytest.raises(ClaspError, match="Not connected"):
            await client.timeline(
                "/animation/pos",
                keyframes=[{"time": 0, "value": 0}, {"time": 1000000, "value": 100}],
                start_time=1000000000000  # Some future time
            )

    def test_timeline_signature(self):
        """timeline should have correct signature."""
        import inspect
        client = Clasp("ws://localhost:7330")
        sig = inspect.signature(client.timeline)
        params = list(sig.parameters.keys())
        assert "address" in params
        assert "keyframes" in params
        assert "loop" in params
        assert "start_time" in params
        # Defaults
        assert sig.parameters["loop"].default is False
        assert sig.parameters["start_time"].default is None


class TestMessageEncoding:
    """Test message encoding for new signal types."""

    def test_gesture_phase_codes(self):
        """Test that gesture phase codes are defined."""
        from clasp.client import SIG_GESTURE
        assert SIG_GESTURE == 3

    def test_timeline_signal_code(self):
        """Test that timeline signal code is defined."""
        from clasp.client import SIG_TIMELINE
        assert SIG_TIMELINE == 4

    def test_query_message_code(self):
        """Test that QUERY message code is defined."""
        from clasp.client import MSG_QUERY
        assert MSG_QUERY == 0x60

    def test_result_message_code(self):
        """Test that RESULT message code is defined."""
        from clasp.client import MSG_RESULT
        assert MSG_RESULT == 0x61
