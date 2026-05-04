import { useAppStore, type OnboardingWizardStep } from "../../state/appStore";
import { useOwnerStore } from "../../state/ownerStore";
import { useSettingsStore } from "../../state/settingsStore";
import {
  persistOnboardingCompleted,
  persistOnboardingStep,
} from "../../state/firstRunPersistence";
import { listCameras, setCamera, getOwnerStatus } from "../../cv/ipc";
import { Button } from "../components/Button";
import { EnrollmentWizard } from "../components/EnrollmentWizard";
import { PermissionExplainer } from "../components/PermissionExplainer";
import { ScreenHeader } from "../components/ScreenHeader";
import { StatusPill } from "../components/StatusPill";
import { Surface } from "../components/Surface";

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
        <ScreenHeader eyebrow="Welcome" title="Set up GlanceGuard">
          <p>GlanceGuard watches your webcam to warn when someone else may be looking at your screen.</p>
        </ScreenHeader>
        <Surface className="panel">
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
        </Surface>
      </div>
    );
  }

  if (step === "camera-explainer") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <ScreenHeader eyebrow="Step 1 of 3" title="Camera access">
          <p>macOS will show its own permission dialog after you continue — not before.</p>
        </ScreenHeader>
        <Surface className="panel">
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
        </Surface>
      </div>
    );
  }

  if (step === "camera-grant") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <ScreenHeader eyebrow="Step 1 of 3" title="Allow the camera">
          <p>The next button opens the camera permission prompt so we can show your preview.</p>
        </ScreenHeader>
        <Surface className="panel">
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
        </Surface>
      </div>
    );
  }

  if (step === "keychain-explainer") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <ScreenHeader eyebrow="Step 2 of 3" title="macOS Keychain">
          <p>When you finish enrollment, the app saves an encrypted owner profile.</p>
        </ScreenHeader>
        <Surface className="panel">
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
        </Surface>
      </div>
    );
  }

  if (step === "enrollment") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <Surface className="panel panel--wide">
          <div className="panel__header panel__header--center">
            <StatusPill tone="info">Step 3 of 3</StatusPill>
          </div>
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
        </Surface>
      </div>
    );
  }

  if (step === "done") {
    return (
      <div className="screen screen--wide onboarding-screen">
        <ScreenHeader eyebrow="Ready" title="You are set">
          <p>Monitoring needs an enrolled owner — you are ready to open the Monitoring tab.</p>
        </ScreenHeader>
        <Surface className="panel">
          <p className="onboarding-screen__done-body">
            Owner enrollment is saved on this Mac. Use the Owner tab if you ever need to replace it.
          </p>
          <div className="actions">
            <Button
              variant="primary"
              onClick={() => {
                void persistOnboardingCompleted(true).then(() => {
                  setOnboardingCompleted(true);
                });
              }}
            >
              Go to app
            </Button>
          </div>
        </Surface>
      </div>
    );
  }

  return null;
};
