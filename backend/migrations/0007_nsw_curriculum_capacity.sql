BEGIN;
-- The replacement has 736 skills, resource links, and bounded per-item practice.
-- Retain finite JSON limits, ownership/RLS and the existing atomic receipt path.
ALTER TABLE wonderwilds.mission_learning_progress
    DROP CONSTRAINT mission_learning_progress_checkpoint_check,
    ADD CONSTRAINT mission_learning_progress_checkpoint_check CHECK (
        jsonb_typeof(checkpoint) = 'object' AND octet_length(checkpoint::text) <= 4194304
    );
ALTER TABLE wonderwilds.mission_learning_command_receipts
    DROP CONSTRAINT mission_learning_command_receipts_response_check,
    ADD CONSTRAINT mission_learning_command_receipts_response_check CHECK (
        jsonb_typeof(response) = 'object' AND octet_length(response::text) <= 2097152
    );
COMMIT;
