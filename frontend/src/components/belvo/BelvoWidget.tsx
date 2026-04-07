/**
 * Belvo Connect widget wrapper component.
 *
 * Loads the Belvo widget script and handles the connection flow.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import type {
  BelvoWidgetSuccessEvent,
  BelvoWidgetErrorEvent,
} from "@/types/belvo";
import { Loader2 } from "lucide-react";

// Belvo widget script URL
const BELVO_WIDGET_SCRIPT = "https://cdn.belvo.io/belvo-widget-1-stable.js";

// Extend Window interface for Belvo SDK
declare global {
  interface Window {
    belvoSDK?: {
      createWidget: (
        accessToken: string,
        options: {
          locale?: string;
          company_name?: string;
          country_codes?: string[];
          callback: (link: string, institution: string) => void;
          onExit?: (data: {
            last_encountered_error?: BelvoWidgetErrorEvent;
          }) => void;
          onEvent?: (event: {
            eventName: string;
            meta_data?: Record<string, unknown>;
          }) => void;
        },
      ) => { build: () => void };
    };
  }
}

interface BelvoWidgetProps {
  accessToken: string;
  onSuccess: (event: BelvoWidgetSuccessEvent) => void;
  onExit?: () => void;
  onError?: (error: BelvoWidgetErrorEvent) => void;
  onEvent?: (eventName: string, metadata?: Record<string, unknown>) => void;
  disabled?: boolean;
}

export function BelvoWidget({
  accessToken,
  onSuccess,
  onExit,
  onError,
  onEvent,
  disabled = false,
}: BelvoWidgetProps) {
  const { t } = useTranslation();
  const [isScriptLoaded, setIsScriptLoaded] = useState(() => !!window.belvoSDK);
  const [isLoading, setIsLoading] = useState(false);
  const callbacksRef = useRef({ onSuccess, onExit, onError, onEvent });

  // Keep callbacks ref updated
  useEffect(() => {
    callbacksRef.current = { onSuccess, onExit, onError, onEvent };
  }, [onSuccess, onExit, onError, onEvent]);

  // Load Belvo widget script
  useEffect(() => {
    if (isScriptLoaded) return;

    const existingScript = document.querySelector(
      `script[src="${BELVO_WIDGET_SCRIPT}"]`,
    );
    if (existingScript) {
      existingScript.addEventListener("load", () => setIsScriptLoaded(true));
      return;
    }

    const script = document.createElement("script");
    script.src = BELVO_WIDGET_SCRIPT;
    script.async = true;
    script.onload = () => setIsScriptLoaded(true);
    script.onerror = () => {
      console.error("Failed to load Belvo widget script");
      onError?.({
        error_code: "SCRIPT_LOAD_ERROR",
        error_message: "Failed to load widget",
      });
    };
    document.body.appendChild(script);
  }, [isScriptLoaded, onError]);

  const handleOpenWidget = useCallback(() => {
    if (!isScriptLoaded || !accessToken || !window.belvoSDK) {
      return;
    }

    setIsLoading(true);

    try {
      window.belvoSDK
        .createWidget(accessToken, {
          locale: "es",
          company_name: "Gasticos",
          // country_codes removed for sandbox - mock banks aren't tagged as Colombian
          // Re-enable for production: country_codes: ['CO'],
          callback: (link: string, institution: string) => {
            setIsLoading(false);
            callbacksRef.current.onSuccess({ link, institution });
          },
          onExit: (data) => {
            setIsLoading(false);
            if (data.last_encountered_error) {
              callbacksRef.current.onError?.(data.last_encountered_error);
            }
            callbacksRef.current.onExit?.();
          },
          onEvent: (event) => {
            callbacksRef.current.onEvent?.(event.eventName, event.meta_data);
          },
        })
        .build();
    } catch (error) {
      setIsLoading(false);
      console.error("Failed to open Belvo widget:", error);
      onError?.({
        error_code: "WIDGET_ERROR",
        error_message: "Failed to open widget",
      });
    }
  }, [isScriptLoaded, accessToken, onError]);

  const isButtonDisabled =
    disabled || !isScriptLoaded || !accessToken || isLoading;

  return (
    <Button
      onClick={handleOpenWidget}
      disabled={isButtonDisabled}
      className="w-full"
      size="lg"
    >
      {isLoading ? (
        <>
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          {t("belvo.connecting")}
        </>
      ) : (
        t("belvo.connectBank")
      )}
    </Button>
  );
}
