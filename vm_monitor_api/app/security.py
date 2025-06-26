import hmac
import hashlib
import base64
from datetime import datetime, timedelta, timezone
from fastapi import Request, HTTPException, status, Header
from typing import Dict

# In-memory store for agent API keys (replace with DB in production)
# Key: instance_id (str), Value: agent_api_key (str)
AGENT_API_KEYS: Dict[str, str] = {}

# Configuration for timestamp validation
TIMESTAMP_VALIDITY_SECONDS = 300  # 5 minutes

def verify_hmac_signature(
    api_key_secret: str,
    timestamp_str: str,
    signature_from_request: str,
    request: Request,
    body_bytes: bytes # Pass the raw body bytes
) -> bool:
    """
    Verifies the HMAC-SHA256 signature of a request.
    """
    try:
        # 1. Validate timestamp format and expiry
        request_timestamp = datetime.fromtimestamp(int(timestamp_str), timezone.utc)
        current_time = datetime.now(timezone.utc)
        if abs((current_time - request_timestamp).total_seconds()) > TIMESTAMP_VALIDITY_SECONDS:
            print(f"Timestamp validation failed: Request ts {request_timestamp}, Server ts {current_time}, Diff {abs((current_time - request_timestamp).total_seconds())}s")
            return False
    except ValueError:
        print("Invalid timestamp format")
        return False # Invalid timestamp format

    # 2. Reconstruct the message to sign
    #    Format: "{timestamp}\n{http_method}\n{request_path}\n{request_body_string}"
    method = request.method.upper()
    path = request.url.path

    # The body_bytes should be the exact string the client signed.
    # If the client sent an empty string for an empty body, use that.
    # If the client sent JSON, it's the JSON string.
    body_str = body_bytes.decode('utf-8')

    message_to_sign = f"{timestamp_str}\n{method}\n{path}\n{body_str}"
    # print(f"Server-side message to sign:\n---\n{message_to_sign}\n---") # For debugging

    # 3. Calculate the signature
    mac = hmac.new(api_key_secret.encode('utf-8'), msg=message_to_sign.encode('utf-8'), digestmod=hashlib.sha256)
    expected_signature_bytes = mac.digest()
    expected_signature_base64 = base64.b64encode(expected_signature_bytes).decode('utf-8')

    # print(f"Client Signature: {signature_from_request}")
    # print(f"Server Signature: {expected_signature_base64}")

    # 4. Compare signatures (use hmac.compare_digest for constant-time comparison)
    return hmac.compare_digest(expected_signature_base64.encode('utf-8'), signature_from_request.encode('utf-8'))


async def authenticate_agent(
    request: Request,
    x_instance_id: str = Header(..., alias="X-Instance-Id"), # Agent should send its ID for key lookup
    x_request_timestamp: str = Header(..., alias="X-Request-Timestamp"),
    x_request_signature: str = Header(..., alias="X-Request-Signature"),
    # Authorization: Optional[str] = Header(None) # Could also use Bearer token if API issues them
):
    """
    Dependency to authenticate agent requests using HMAC signature.
    The agent must send its `instance_id` in a header (e.g., X-Instance-Id)
    so the server can look up its specific API key.
    """
    agent_secret_key = AGENT_API_KEYS.get(x_instance_id)

    if not agent_secret_key:
        print(f"Authentication failed: No API key found for instance_id '{x_instance_id}'")
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid instance ID or agent not registered with an API key.",
        )

    # Read the raw request body. This is crucial as it must match what the client signed.
    # FastAPI normally consumes the body, so we need to get it before Pydantic parsing for the endpoint.
    body_bytes = await request.body() # This reads the raw body

    if not verify_hmac_signature(
        api_key_secret=agent_secret_key,
        timestamp_str=x_request_timestamp,
        signature_from_request=x_request_signature,
        request=request,
        body_bytes=body_bytes
    ):
        print(f"Authentication failed: HMAC signature verification failed for instance_id '{x_instance_id}'")
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid signature or timestamp.",
            headers={"WWW-Authenticate": "Signature"},
        )
    print(f"Agent {x_instance_id} authenticated successfully.")
    # The body_bytes have been consumed by `await request.body()`.
    # If the endpoint needs to parse it with Pydantic, it might need to be "re-fed".
    # A common pattern is to parse it here and pass it as a request state or attribute.
    # For now, endpoints will re-parse, but FastAPI handles this if `request.body()` was called.
    return {"instance_id": x_instance_id, "raw_body_bytes": body_bytes} # Pass body if needed, or just auth success