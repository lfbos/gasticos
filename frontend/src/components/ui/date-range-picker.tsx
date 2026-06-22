/**
 * Date range picker with presets.
 */

import * as React from "react";
import {
  format,
  subDays,
  subMonths,
  subYears,
  startOfMonth,
  startOfQuarter,
  startOfYear,
} from "date-fns";
import { es } from "date-fns/locale";
import { Calendar as CalendarIcon, X } from "lucide-react";
import type { DateRange } from "react-day-picker";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface DateRangePickerProps {
  value: DateRange | undefined;
  onChange: (range: DateRange | undefined) => void;
  className?: string;
}

const presets = [
  {
    label: "Últimos 7 días",
    getValue: () => ({
      from: subDays(new Date(), 7),
      to: new Date(),
    }),
  },
  {
    label: "Últimos 30 días",
    getValue: () => ({
      from: subDays(new Date(), 30),
      to: new Date(),
    }),
  },
  {
    label: "Últimos 90 días",
    getValue: () => ({
      from: subDays(new Date(), 90),
      to: new Date(),
    }),
  },
  {
    label: "Último mes",
    getValue: () => ({
      from: startOfMonth(subMonths(new Date(), 1)),
      to: subDays(startOfMonth(new Date()), 1),
    }),
  },
  {
    label: "Este mes",
    getValue: () => ({
      from: startOfMonth(new Date()),
      to: new Date(),
    }),
  },
  {
    label: "Este trimestre",
    getValue: () => ({
      from: startOfQuarter(new Date()),
      to: new Date(),
    }),
  },
  {
    label: "Este año",
    getValue: () => ({
      from: startOfYear(new Date()),
      to: new Date(),
    }),
  },
  {
    label: "Último año",
    getValue: () => ({
      from: startOfYear(subYears(new Date(), 1)),
      to: subDays(startOfYear(new Date()), 1),
    }),
  },
];

export function DateRangePicker({
  value,
  onChange,
  className,
}: DateRangePickerProps) {
  const [open, setOpen] = React.useState(false);
  const [tempRange, setTempRange] = React.useState<DateRange | undefined>(
    undefined,
  );

  // Sync tempRange with value when popover opens
  React.useEffect(() => {
    if (open) {
      setTempRange(value);
    }
  }, [open, value]);

  const handlePresetClick = (preset: (typeof presets)[0]) => {
    const range = preset.getValue();
    onChange(range);
    setOpen(false);
  };

  const handleClear = () => {
    onChange(undefined);
    setTempRange(undefined);
    setOpen(false);
  };

  const handleApply = () => {
    if (tempRange?.from) {
      onChange({
        from: tempRange.from,
        to: tempRange.to || tempRange.from,
      });
    }
    setOpen(false);
  };

  const handleCancel = () => {
    setTempRange(value);
    setOpen(false);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className={cn(
            "justify-start text-left font-normal",
            !value && "text-muted-foreground",
            className,
          )}
        >
          <CalendarIcon className="mr-2 h-4 w-4" />
          {value?.from ? (
            value.to ? (
              <>
                {format(value.from, "dd MMM yyyy", { locale: es })} -{" "}
                {format(value.to, "dd MMM yyyy", { locale: es })}
              </>
            ) : (
              format(value.from, "dd MMM yyyy", { locale: es })
            )
          ) : (
            <span>Seleccionar fechas</span>
          )}
          {value && (
            <X
              className="ml-2 h-4 w-4 opacity-50 hover:opacity-100"
              onClick={(e) => {
                e.stopPropagation();
                handleClear();
              }}
            />
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="end">
        <div className="flex">
          {/* Presets sidebar */}
          <div className="hidden md:flex flex-col border-r py-2 w-40">
            <div className="px-2 pb-2 mb-2 border-b">
              <span className="text-xs font-medium text-muted-foreground">
                Rangos rápidos
              </span>
            </div>
            <div className="space-y-0.5 px-1">
              {presets.map((preset) => (
                <button
                  key={preset.label}
                  type="button"
                  className="w-full text-left px-2 py-1.5 text-sm rounded hover:bg-accent transition-colors"
                  onClick={() => handlePresetClick(preset)}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>
          {/* Calendar and actions */}
          <div className="flex flex-col">
            <div className="p-3">
              <Calendar
                mode="range"
                defaultMonth={tempRange?.from || new Date()}
                selected={tempRange}
                onSelect={setTempRange}
                numberOfMonths={2}
              />
            </div>
            {/* Footer with apply button */}
            <div className="flex items-center justify-between border-t p-3 gap-2">
              <div className="text-sm text-muted-foreground">
                {tempRange?.from ? (
                  tempRange.to ? (
                    <>
                      {format(tempRange.from, "dd/MM/yyyy")} -{" "}
                      {format(tempRange.to, "dd/MM/yyyy")}
                    </>
                  ) : (
                    <>{format(tempRange.from, "dd/MM/yyyy")} - Selecciona fin</>
                  )
                ) : (
                  <>Selecciona un rango</>
                )}
              </div>
              <div className="flex gap-2">
                <Button variant="ghost" size="sm" onClick={handleCancel}>
                  Cancelar
                </Button>
                <Button
                  size="sm"
                  onClick={handleApply}
                  disabled={!tempRange?.from}
                >
                  Aplicar
                </Button>
              </div>
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
