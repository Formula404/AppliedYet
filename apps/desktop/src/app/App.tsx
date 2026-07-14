import { Navigate, Route, Routes } from "react-router-dom";
import AppShell from "../components/AppShell";
import HomePage from "../pages/HomePage";
import ApplicationsPage from "../pages/ApplicationsPage";
import EmailsPage from "../pages/EmailsPage";
import FeaturePages from "../pages/FeaturePages";

export default function App() {
  return <Routes><Route element={<AppShell />}>
    <Route index element={<HomePage />} />
    <Route path="applications" element={<ApplicationsPage />} />
    <Route path="emails" element={<EmailsPage />} />
    <Route path="preparation" element={<FeaturePages kind="preparation" />} />
    <Route path="mock-interview" element={<FeaturePages kind="mock" />} />
    <Route path="reviews" element={<FeaturePages kind="reviews" />} />
    <Route path="question-bank" element={<FeaturePages kind="questions" />} />
    <Route path="offers" element={<FeaturePages kind="offers" />} />
    <Route path="analytics" element={<FeaturePages kind="analytics" />} />
    <Route path="settings" element={<FeaturePages kind="settings" />} />
    <Route path="*" element={<Navigate to="/" replace />} />
  </Route></Routes>;
}
