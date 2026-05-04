import { useState } from "react";
import {
  isValidLicenseKeyFormat,
  persistLicenseGatePassed,
} from "../../state/firstRunPersistence";
import { useAppStore } from "../../state/appStore";
import { PrivacyOnboardingFooter } from "../components/PrivacyOnboardingFooter";

/**
 * Phase 6: collects and persists gate passage; cryptographic license verification / JWT (Phase 11) is not wired yet.
 * Production builds still require a well-formed key — there is no silent bypass. Development builds may use the explicit skip control (import.meta.env.DEV only).
 */
export const LicenseGateScreen = () => {
  const setLicenseGatePassed = useAppStore((s) => s.setLicenseGatePassed);
  const [licenseKey, setLicenseKey] = useState("");
  const [error, setError] = useState<string | undefined>();

  const submit = async () => {
    const trimmed = licenseKey.trim();
    if (!isValidLicenseKeyFormat(trimmed)) {
      setError(
        "Use the license key from your purchase email (format SP#-XXXX-XXXX-XXXX). Online activation arrives in Phase 11.",
      );
      return;
    }
    setError(undefined);
    await persistLicenseGatePassed(true);
    setLicenseGatePassed(true);
  };

  const skipDevOnly = async () => {
    if (!import.meta.env.DEV) {
      return;
    }
    setError(undefined);
    await persistLicenseGatePassed(true);
    setLicenseGatePassed(true);
  };

  return (
    <div className="screen license-gate">
      <div className="screen__header">
        <h2>Enter your license</h2>
        <p>
          Screen Peek Alert is a one-time purchase. There is no free trial — refunds are handled on the store side
          (see the product page).
        </p>
      </div>

      <div className="panel">
        <label className="field">
          <span className="field__label">License key</span>
          <input
            className="field__input"
            autoComplete="off"
            spellCheck={false}
            aria-label="License key"
            placeholder="SP1-XXXX-XXXX-XXXX"
            value={licenseKey}
            onChange={(e) => {
              setLicenseKey(e.target.value);
              setError(undefined);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                void submit();
              }
            }}
          />
        </label>

        {error ? <div className="status__error">{error}</div> : null}

        <div className="actions">
          <button type="button" className="button button--primary" onClick={() => void submit()}>
            Continue
          </button>
          {import.meta.env.DEV ? (
            <button type="button" className="button button--ghost" onClick={() => void skipDevOnly()}>
              Skip for local dev (not in production)
            </button>
          ) : null}
        </div>

        <PrivacyOnboardingFooter />
      </div>
    </div>
  );
};
