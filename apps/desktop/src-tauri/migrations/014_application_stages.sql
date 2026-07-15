-- Consolidate legacy paused/ended states into the product's talent-pool state.
UPDATE applications SET current_stage='进入人才库' WHERE current_stage IN ('流程暂停','流程结束','暂停/结束');
UPDATE tasks SET application_stage='进入人才库' WHERE application_stage IN ('流程暂停','流程结束','暂停/结束');
UPDATE application_events SET stage_before='进入人才库' WHERE stage_before IN ('流程暂停','流程结束','暂停/结束');
UPDATE application_events SET stage_after='进入人才库' WHERE stage_after IN ('流程暂停','流程结束','暂停/结束');
