import { useState } from "react";
import {
  isValidLicenseKeyFormat,
  persistLicenseGatePassed,
} from "../../state/firstRunPersistence";
import { useLicenseStore } from "../../state/licenseStore";
import { PrivacyOnboardingFooter } from "../components/PrivacyOnboardingFooter";

/**
 * Temporary license placeholder: only validates the local key format until real
 * cryptographic activation / JWT verification is wired.
 */
export const LicenseGateScreen = () => {
  const setLicenseGatePassed = useLicenseStore((s) => s.setLicenseGatePassed);
  const [licenseKey, setLicenseKey] = useState("");
  const [error, setError] = useState<string | undefined>();

  const submit = async () => {
    const trimmed = licenseKey.trim();
    if (!isValidLicenseKeyFormat(trimmed)) {
      setError(
        "This placeholder only accepts the test format GG#-XXXX-XXXX-XXXX. Real online activation is not wired yet.",
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
        <h2>License activation placeholder</h2>
        <p>
          Real license validation is not connected in this build yet. This screen only keeps
          the onboarding shape ready while activation is wired.
        </p>
      </div>

      <div className="panel">
        <div className="license-gate__notice" role="status">
          <strong>Not production licensing.</strong>
          <span>
            For now, this checks only the key format and stores a local “passed” flag. The
            next licensing pass must replace this with server-backed activation and signed
            offline state.
          </span>
        </div>

        <label className="field">
          <span className="field__label">Temporary test license key</span>
          <input
            className="field__input"
            autoComplete="off"
            spellCheck={false}
            aria-label="Temporary test license key"
            placeholder="GG1-XXXX-XXXX-XXXX"
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
            Continue with placeholder key
          </button>
          {import.meta.env.DEV ? (
            <button type="button" className="button button--ghost" onClick={() => void skipDevOnly()}>
              Skip placeholder for testing
            </button>
          ) : null}
        </div>

        <PrivacyOnboardingFooter />
      </div>
    </div>
  );
};
