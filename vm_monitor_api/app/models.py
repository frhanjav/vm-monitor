from pydantic import BaseModel, Field
from typing import List, Dict, Any, Optional
from datetime import datetime
import uuid

# --- Agent Sent Payloads ---
class AgentRegistrationPayload(BaseModel):
    instance_id: uuid.UUID # Use UUID type for validation
    instance_name: str
    cloud_provider: str
    # The agent generates its API key and should send it during registration
    # This key will be used by the server to validate future signatures from this agent
    agent_api_key: str = Field(..., description="The API key generated by the agent, to be stored by the server")

class CPUMetrics(BaseModel):
    usage_percent: float
    core_count: int
    per_core_usage: List[float]

class MemoryMetrics(BaseModel):
    total_memory: int
    used_memory: int
    available_memory: int
    total_swap: int
    used_swap: int

class DiskMetric(BaseModel):
    name: str
    mount_point: str
    total_space: int
    available_space: int
    filesystem: str
    total_written_bytes: int
    total_read_bytes: int

class NetworkMetric(BaseModel):
    interface_name: str
    received_bytes_total: int
    transmitted_bytes_total: int

class SystemInfo(BaseModel):
    hostname: str
    os_name: str
    os_version: str
    kernel_version: str
    uptime: int

class SystemMetricsPayload(BaseModel):
    timestamp: datetime # Pydantic will parse ISO datetime strings
    instance_id: uuid.UUID
    cpu_metrics: CPUMetrics
    memory_metrics: MemoryMetrics
    disk_metrics: List[DiskMetric]
    network_metrics: List[NetworkMetric]
    system_info: SystemInfo

class MetricsBatchWrapper(BaseModel):
    metrics: List[SystemMetricsPayload]

class HeartbeatPayload(BaseModel):
    instance_id: uuid.UUID

# --- API Responses ---
class AgentRegistrationResponse(BaseModel):
    message: str
    instance_id: uuid.UUID
    # Maybe return some server-side info if needed
    # server_issued_token: Optional[str] = None # If server issues its own token

class MessageResponse(BaseModel):
    message: str

class HealthResponse(BaseModel):
    status: str
    message: str
    timestamp: datetime

# --- Internal Storage Models (for this mock in-memory version) ---
class StoredAgent(BaseModel):
    instance_id: uuid.UUID
    instance_name: str
    cloud_provider: str
    agent_api_key: str # Store the agent's key for signature validation
    registered_at: datetime
    last_heartbeat_at: Optional[datetime] = None

class StoredMetricsBatch(BaseModel):
    received_at: datetime
    instance_id: uuid.UUID
    metrics: List[SystemMetricsPayload]