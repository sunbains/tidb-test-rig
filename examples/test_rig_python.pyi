"""
Type stub for test_rig_python module.
This file helps IDEs understand the types without requiring the actual module.
"""

from typing import Any, Optional

class PyStateHandler:
    """Base class for Python state handlers."""
    def __init__(self) -> None: ...
    def enter(self, context: 'PyStateContext') -> str: ...
    def execute(self, context: 'PyStateContext') -> str: ...
    def exit(self, context: 'PyStateContext') -> None: ...

class PyStateContext:
    """Context for state handlers."""
    host: Optional[str]
    port: Optional[int]
    username: Optional[str]
    password: Optional[str]
    database: Optional[str]
    connection: Optional[Any]
    
    def __init__(self, host: Optional[str] = None, 
                 database: Optional[str] = None, 
                 connection: Optional[Any] = None) -> None: ...

class PyState:
    """State enum for the state machine."""
    @staticmethod
    def initial() -> str: ...
    @staticmethod
    def completed() -> str: ...
    @staticmethod
    def connecting() -> str: ...
    @staticmethod
    def testing_connection() -> str: ...
    @staticmethod
    def testing_isolation() -> str: ...
    @staticmethod
    def checking_import_jobs() -> str: ...
    @staticmethod
    def showing_import_job_details() -> str: ... 