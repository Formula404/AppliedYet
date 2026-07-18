import { useEffect, useState } from "react";
import { X } from "lucide-react";
import type { Application } from "../types";
import type { CreateApplicationInput } from "../services/applications";
import { listResumeProfiles, type ResumeProfile } from "../services/resumes";

export interface NewApplicationDefaults {
  companyName?: string;
  positionTitle?: string;
  appliedAt?: string;
  channel?: string;
}

interface NewApplicationDialogProps {
  defaults?: NewApplicationDefaults;
  description?: string;
  emailStage?: string;
  requireAppliedAt?: boolean;
  saving: boolean;
  onClose: () => void;
  onSubmit: (input: CreateApplicationInput) => Promise<Application>;
  onError: (reason: unknown) => void;
}

export function NewApplicationDialog({
  defaults,
  description = "先记录核心信息，稍后可继续完善岗位档案",
  emailStage,
  requireAppliedAt = false,
  saving,
  onClose,
  onSubmit,
  onError,
}: NewApplicationDialogProps) {
  const [resumes, setResumes] = useState<ResumeProfile[]>([]);
  const [resumeError, setResumeError] = useState("");

  useEffect(() => {
    listResumeProfiles()
      .then((items) => setResumes(items.filter((item) => !item.archivedAt)))
      .catch((reason) => setResumeError(String(reason)));
  }, []);

  return (
    <div className="modal-backdrop application-modal-backdrop">
      <div className="dialog application-dialog" role="dialog" aria-modal="true" aria-labelledby="new-application-title">
        <div className="dialog-head">
          <div><h2 id="new-application-title">新增投递</h2><p>{description}</p></div>
          <button type="button" onClick={onClose} disabled={saving} aria-label="关闭"><X size={19} /></button>
        </div>
        {emailStage && <div className="email-import-notice"><strong>来自招聘邮件 · {emailStage}</strong><span>保存后会关联原邮件，并把识别到的阶段写入流程时间线。</span></div>}
        {resumeError && <div className="settings-notice">简历列表读取失败：{resumeError}。仍可先创建投递。</div>}
        <form onSubmit={async (event) => {
          event.preventDefault();
          const data = new FormData(event.currentTarget);
          try {
            await onSubmit({
              companyName: String(data.get("companyName") || ""),
              companyShortName: String(data.get("companyShortName") || "") || undefined,
              industry: String(data.get("industry") || "") || undefined,
              companyType: String(data.get("companyType") || "") || undefined,
              website: String(data.get("website") || "") || undefined,
              companyNotes: String(data.get("companyNotes") || "") || undefined,
              positionTitle: String(data.get("positionTitle") || ""),
              department: String(data.get("department") || "") || undefined,
              location: String(data.get("location") || ""),
              recruitmentType: String(data.get("recruitmentType") || "") || undefined,
              jobCode: String(data.get("jobCode") || "") || undefined,
              sourceUrl: String(data.get("sourceUrl") || "") || undefined,
              channel: String(data.get("channel") || ""),
              appliedAt: String(data.get("appliedAt") || "") || undefined,
              priority: Number(data.get("priority") || 2),
              jdRaw: String(data.get("jdRaw") || ""),
              resumeProfileId: String(data.get("resumeProfileId") || "") || undefined,
            });
          } catch (reason) {
            onError(reason);
          }
        }}>
          <div className="form-grid">
            <label><span>公司名称 *</span><input name="companyName" required defaultValue={defaults?.companyName} placeholder="例如：蚂蚁集团" /></label>
            <label><span>公司简称</span><input name="companyShortName" placeholder="例如：蚂蚁" /></label>
            <label><span>行业</span><input name="industry" placeholder="例如：互联网金融" /></label>
            <label><span>公司性质</span><input name="companyType" placeholder="例如：民营企业" /></label>
            <label><span>岗位名称 *</span><input name="positionTitle" required defaultValue={defaults?.positionTitle} placeholder="例如：后端开发工程师" /></label>
            <label><span>部门</span><input name="department" placeholder="例如：基础架构部" /></label>
            <label><span>工作地点</span><input name="location" placeholder="杭州" /></label>
            <label><span>招聘类型</span><select name="recruitmentType"><option value="">未设置</option><option>校招</option><option>实习</option><option>社招</option></select></label>
            <label><span>岗位编号</span><input name="jobCode" /></label>
            <label><span>投递渠道</span><select name="channel" defaultValue={defaults?.channel ?? "招聘官网"}><option>招聘官网</option><option>Boss 直聘</option><option>内推</option><option>邮件识别</option><option>其他</option></select></label>
            <label><span>投递日期{requireAppliedAt ? " *" : ""}</span><input name="appliedAt" type="date" required={requireAppliedAt} defaultValue={defaults?.appliedAt ?? new Date().toLocaleDateString("en-CA")} /></label>
            <label><span>优先级</span><select name="priority" defaultValue="2"><option value="3">高</option><option value="2">中</option><option value="1">普通</option></select></label>
            <label><span>使用简历</span><select key={resumes.map((item) => item.id).join("|")} name="resumeProfileId" defaultValue={resumes.find((item) => item.isPrimary)?.id ?? ""}><option value="">暂不关联</option>{resumes.map((resume) => <option key={resume.id} value={resume.id}>{resume.name}{resume.targetDirection ? ` · ${resume.targetDirection}` : ""}{resume.isPrimary ? "（默认）" : ""}</option>)}</select></label>
            <label><span>公司官网</span><input name="website" type="url" placeholder="https://" /></label>
            <label className="full"><span>招聘链接</span><input name="sourceUrl" type="url" placeholder="https://" /></label>
            <label className="full"><span>JD 原文</span><textarea name="jdRaw" rows={5} placeholder="粘贴岗位描述，方便 AI 帮你做面试准备" /></label>
            <label className="full"><span>公司备注</span><textarea name="companyNotes" rows={3} placeholder="记录团队、业务或沟通信息" /></label>
          </div>
          <div className="dialog-actions">
            <button type="button" className="button button--secondary" onClick={onClose} disabled={saving}>取消</button>
            <button className="button button--primary" disabled={saving}>{saving ? "保存中…" : "保存投递"}</button>
          </div>
        </form>
      </div>
    </div>
  );
}
