import hmac
import hashlib
import base64
from datetime import datetime, timezone
from fastapi import Request, HTTPException, status, Header
from typing import Dict

AGENT_API_KEYS: Dict[str, str] = {}

TIMESTAMP_VALIDITY_SECONDS = 300

def verify_hmac_signature(
    api_key_secret: str,
    timestamp_str: str,
    signature_from_request: str,
    request: Request,
    body_bytes: bytes
) -> bool:
    """
    Verifies the HMAC-SHA256 signature of a request.
    """
    try:
        request_timestamp = datetime.fromtimestamp(int(timestamp_str), timezone.utc)
        current_time = datetime.now(timezone.utc)
        if abs((current_time - request_timestamp).total_seconds()) > TIMESTAMP_VALIDITY_SECONDS:
            print(f"Timestamp validation failed: Request ts {request_timestamp}, Server ts {current_time}, Diff {abs((current_time - request_timestamp).total_seconds())}s")
            return False
    except ValueError:
        print("Invalid timestamp format")
        return False

    method = request.method.upper()
    path = request.url.path

    body_str = body_bytes.decode('utf-8')

    message_to_sign = f"{timestamp_str}\n{method}\n{path}\n{body_str}"

    mac = hmac.new(api_key_secret.encode('utf-8'), msg=message_to_sign.encode('utf-8'), digestmod=hashlib.sha256)
    expected_signature_bytes = mac.digest()
    expected_signature_base64 = base64.b64encode(expected_signature_bytes).decode('utf-8')

    return hmac.compare_digest(expected_signature_base64.encode('utf-8'), signature_from_request.encode('utf-8'))


async def authenticate_agent(
    request: Request,
    x_instance_id: str = Header(..., alias="X-Instance-Id"),
    x_request_timestamp: str = Header(..., alias="X-Request-Timestamp"),
    x_request_signature: str = Header(..., alias="X-Request-Signature"),
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

    body_bytes = await request.body()

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
    return {"instance_id": x_instance_id, "raw_body_bytes": body_bytes}