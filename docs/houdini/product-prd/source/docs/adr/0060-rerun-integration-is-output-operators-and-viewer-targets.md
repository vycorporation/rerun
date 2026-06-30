# Rerun integration is output operators and viewer targets

Houdini Clone models Rerun integration as graph-visible output operators and viewer targets rather than as a special viewport backend. Rerun output operators send compatible typed graph outputs or commands to Rerun viewer targets, keeping Rerun-specific behavior explicit, inspectable, replaceable, and compatible with graph orchestration across multiple tools.
