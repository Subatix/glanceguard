import { useAppStore, type OnboardingWizardStep } from "../../state/appStore";
import { useOwnerStore } from "../../state/ownerStore";
import { useSettingsStore } from "../../state/settingsStore";
import {
  persistOnboardingCompleted,
  persistOnboardingStep,
} from "../../state/firstRunPersistence";
import { listCameras, setCamera, getOwnerStatus } from "../../cv/ipc";
import { EnrollmentWizard } from "../components/EnrollmentWizard";
import { PermissionExplainer } from "../components/PermissionExplainer";

export const OnboardingScreen = () => {
  const step = useAppStore((s) => s.onboarding.step);
  const error = useAppStore((s) => s.error);
  const setOnboardingStep = useAppStore((s) => s.setOnboardingStep);
  const setOnboardingCompleted = useAppStore((s) => s.setOnboardingCompleted);
  const setError = useAppStore((s) => s.setError);
  const setCameras = useSettingsStore((s) => s.setCameras);
  const setSettings = useSettingsStore((s) => s.setSettings);
  const setOwnerEnrolled = useOwnerStore((s) => s.setOwnerEnrolled);

  const advance = async (next: OnboardingWizardStep) => {
    setOnboardingStep(next);
    await persistOnboardingStep(next);
  };

  if (step === "welcome") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="screen__header">
          <h2>Welcome</h2>
          <p>GlanceGuard watches your webcam to warn when someone else may be looking at your screen.</p>
        </div>
        <div className="panel">
          <PermissionExplainer
            title="Before we start"
            body={
              <p>
                We will ask for camera access so you can enroll your face, then save the profile securely on this Mac.
                You will see a short explanation before each system prompt.
              </p>
            }
            primaryLabel="Continue"
            onPrimary={() => void advance("camera-explainer")}
          />
        </div>
      </div>
    );
  }

  if (step === "camera-explainer") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="screen__header">
          <h2>Camera access</h2>
          <p>macOS will show its own permission dialog after you continue — not before.</p>
        </div>
        <div className="panel">
          <PermissionExplainer
            title="Why the camera"
            body={
              <p>
                The app needs the camera to see who is in front of your laptop. Video is processed on device only;
                nothing is uploaded.
              </p>
            }
            primaryLabel="Continue"
            onPrimary={() => void advance("camera-grant")}
            secondaryLabel="Back"
            onSecondary={() => void advance("welcome")}
          />
        </div>
      </div>
    );
  }

  if (step === "camera-grant") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="screen__header">
          <h2>Allow the camera</h2>
          <p>The next button triggers the real browser / system camera prompt (same path as live preview in Owner setup).</p>
        </div>
        <div className="panel">
          <PermissionExplainer
            title="Camera permission"
            body={
              <p>
                When the system dialog appears, choose <strong>Allow</strong> so we can capture enrollment frames. You
                can change this later in System Settings.
              </p>
            }
            primaryLabel="Request camera access"
            onPrimary={() => {
              setError(undefined);
              void (async () => {
                try {
                  const stream = await navigator.mediaDevices.getUserMedia({
                    video: { width: { ideal: 1280 }, height: { ideal: 720 }, facingMode: "user" },
                    audio: false,
                  });
                  stream.getTracks().forEach((t) => t.stop());
                  const cams = await listCameras();
                  if (cams.length === 0) {
                    setError("No cameras detected. Connect a camera and try again.");
                    return;
                  }
                  setCameras(cams);
                  const first = cams[0];
                  if (!first) {
                    setError("No cameras detected. Connect a camera and try again.");
                    return;
                  }
                  const nextSettings = await setCamera(first.id);
                  setSettings(nextSettings);
                  await advance("keychain-explainer");
                } catch (err) {
                  setError(String(err));
                }
              })();
            }}
            secondaryLabel="Back"
            onSecondary={() => void advance("camera-explainer")}
          />
          {error ? <div className="status__error onboarding-screen__inline-error">{error}</div> : null}
        </div>
      </div>
    );
  }

  if (step === "keychain-explainer") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="screen__header">
          <h2>macOS Keychain</h2>
          <p>When you finish enrollment, the app saves an encrypted owner profile.</p>
        </div>
        <div className="panel">
          <PermissionExplainer
            title="Encryption key in Keychain"
            body={
              <p>
                A random key used to encrypt your owner embedding is stored in the Keychain. macOS may ask you to allow
                Keychain access when saving — that is expected and required once on this Mac.
              </p>
            }
            primaryLabel="Continue to enrollment"
            onPrimary={() => void advance("enrollment")}
            secondaryLabel="Back"
            onSecondary={() => void advance("camera-grant")}
          />
        </div>
      </div>
    );
  }

  if (step === "enrollment") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="panel panel--wide">
          {error ? <div className="status__error onboarding-screen__inline-error">{error}</div> : null}
          <EnrollmentWizard
            onComplete={() => {
              void getOwnerStatus()
                .then((enrolled) => {
                  setOwnerEnrolled(enrolled);
                  return advance("done");
                })
                .catch((err) => setError(String(err)));
            }}
            onClearError={() => setError(undefined)}
            onError={(msg) => setError(msg)}
          />
        </div>
      </div>
    );
  }

  if (step === "done") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <div className="screen__header">
          <h2>You are set</h2>
          <p>Monitoring needs an enrolled owner — you are ready to open the Monitoring tab.</p>
        </div>
        <div className="panel">
          <p className="onboarding-screen__done-body">
            Owner enrollment is saved on this Mac. You can re-run owner setup from the Owner tab if you ever need to
            replace it.
          </p>
          <div className="actions">
            <button
              type="button"
              className="button button--primary"
              onClick={() => {
                void persistOnboardingCompleted(true).then(() => {
                  setOnboardingCompleted(true);
                });
              }}
            >
              Go to app
            </button>
          </div>
        </div>
      </div>
    );
  }

  return null;
};
