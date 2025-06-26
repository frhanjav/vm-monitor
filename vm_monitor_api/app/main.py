from fastapi import FastAPI, HTTPException, Depends, Body, Request, status
from fastapi.middleware.cors import CORSMiddleware
from typing import List, Dict
from datetime import datetime, timezone
from . import models
import uuid

from . import models
from . import security

# In-memory "database" for the mock server
# Using Pydantic models for stored data for consistency
db_agents: Dict[uuid.UUID, models.StoredAgent] = {}
db_metrics: Dict[uuid.UUID, List[models.StoredMetricsBatch]] = {} # instance_id -> list of batches

app = FastAPI(
    title="VM Monitor API",
    description="API for collecting metrics from vm-monitor agents.",
    version="0.1.0"
)

# CORS middleware (allow all for local dev, restrict in production)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Or specify your frontend URL like "http://localhost:5173"
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/v1/health", response_model=models.HealthResponse, tags=["General"])
async def health_check():
    """
    Check the health of the API.
    """
    return {
        "status": "ok",
        "message": "API is healthy and running",
        "timestamp": datetime.now(timezone.utc)
    }

# Agent Registration - This endpoint should NOT require the full HMAC auth
# because the agent doesn't have a key known to the server yet, or is establishing it.
# It might have a simpler form of auth or be more open if it's the key exchange point.
# For this version, let's assume registration sends the agent-generated key.
@app.post("/v1/agent/register", response_model=models.AgentRegistrationResponse, status_code=status.HTTP_201_CREATED, tags=["Agent"])
async def register_agent(payload: models.AgentRegistrationPayload):
    """
    Register a new vm-monitor agent.
    The agent sends its self-generated API key, which the server stores.
    """
    if payload.instance_id in db_agents:
        # Allow re-registration to update details or API key
        print(f"Agent {payload.instance_id} is re-registering.")
    else:
        print(f"New agent registration: {payload.instance_id}")

    stored_agent = models.StoredAgent(
        instance_id=payload.instance_id,
        instance_name=payload.instance_name,
        cloud_provider=payload.cloud_provider,
        agent_api_key=payload.agent_api_key, # Store the key provided by the agent
        registered_at=datetime.now(timezone.utc)
    )
    db_agents[payload.instance_id] = stored_agent
    security.AGENT_API_KEYS[str(payload.instance_id)] = payload.agent_api_key # Update security key store

    db_metrics.setdefault(payload.instance_id, []) # Initialize metrics list

    print(f"Agent '{payload.instance_name}' ({payload.instance_id}) registered with API key prefix: {payload.agent_api_key[:8]}...")
    return {
        "message": "Agent registered successfully",
        "instance_id": payload.instance_id
    }

# For subsequent requests, we use the authentication dependency
AuthenticatedAgent = Depends(security.authenticate_agent)

@app.post("/v1/agent/metrics", response_model=models.MessageResponse, status_code=status.HTTP_202_ACCEPTED, tags=["Agent"])
async def receive_metrics(
    # The body is now parsed from `authenticated_agent_data["raw_body_bytes"]`
    # because `authenticate_agent` consumed it.
    # Alternatively, `authenticate_agent` could parse it and add it to request state.
    # For simplicity here, we'll re-parse. Pydantic handles this.
    payload_wrapper: models.MetricsBatchWrapper, # FastAPI will parse from the re-read body
    authenticated_agent_data: dict = AuthenticatedAgent # This runs auth
):
    """
    Receive a batch of metrics from an authenticated agent.
    """
    instance_id_from_auth = uuid.UUID(authenticated_agent_data["instance_id"])
    metrics_batch = payload_wrapper.metrics

    if not metrics_batch:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Empty metrics batch received.")

    # Validate that all metrics in the batch match the authenticated instance_id
    for metric in metrics_batch:
        if metric.instance_id != instance_id_from_auth:
            raise HTTPException(
                status_code=status.HTTP_400_BAD_REQUEST,
                detail=f"Mismatch in instance_id in metrics payload ({metric.instance_id}) and authenticated agent ({instance_id_from_auth})."
            )

    batch_to_store = models.StoredMetricsBatch(
        received_at=datetime.now(timezone.utc),
        instance_id=instance_id_from_auth,
        metrics=metrics_batch
    )
    db_metrics.setdefault(instance_id_from_auth, []).append(batch_to_store)

    print(f"Received metrics batch (count: {len(metrics_batch)}) for agent {instance_id_from_auth}.")
    return {"message": f"Metrics batch for {instance_id_from_auth} accepted."}


@app.post("/v1/agent/heartbeat", response_model=models.MessageResponse, tags=["Agent"])
async def agent_heartbeat(
    payload: models.HeartbeatPayload, # Parsed from re-read body
    authenticated_agent_data: dict = AuthenticatedAgent
):
    """
    Receive a heartbeat from an authenticated agent.
    """
    instance_id_from_auth = uuid.UUID(authenticated_agent_data["instance_id"])

    if payload.instance_id != instance_id_from_auth:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Mismatch in instance_id in heartbeat payload and authenticated agent."
        )

    if instance_id_from_auth in db_agents:
        db_agents[instance_id_from_auth].last_heartbeat_at = datetime.now(timezone.utc)
        print(f"Heartbeat received from agent {instance_id_from_auth}.")
        return {"message": "Heartbeat acknowledged"}
    else:
        # Should not happen if auth passed, but good to check
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Agent not found for heartbeat.")


# --- Admin/Debug Endpoints (no auth for this mock, add auth in production) ---
@app.get("/admin/agents", response_model=Dict[uuid.UUID, models.StoredAgent], tags=["Admin"])
async def get_all_agents():
    """
    (Admin) Get all registered agents.
    """
    return db_agents

@app.get("/admin/metrics/{instance_id_str}", response_model=List[models.StoredMetricsBatch], tags=["Admin"])
async def get_metrics_for_agent_admin(instance_id_str: str):
    """
    (Admin) Get all metric batches for a specific agent.
    """
    try:
        instance_id = uuid.UUID(instance_id_str)
        if instance_id not in db_metrics:
            raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="No metrics found for this instance ID.")
        return db_metrics[instance_id]
    except ValueError:
        raise HTTPException(status_code=status.HTTP_400_BAD_REQUEST, detail="Invalid instance_id format.")

@app.on_event("startup")
async def startup_event():
    print("VM Monitor API starting up...")
    # Load any initial config or connect to DB here if needed

@app.on_event("shutdown")
async def shutdown_event():
    print("VM Monitor API shutting down...")