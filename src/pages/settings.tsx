import { useState, useEffect } from 'react';
import * as KindeAuth from '@kinde-oss/kinde-auth-react';
import {
  ArrowRight,
  Cloud,
  Settings2,
  ChevronDown,
  MapPin
} from 'lucide-react';
import { useForm } from 'react-hook-form';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage
} from '@/components/ui/form';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogClose
} from '@/components/ui/dialog';
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerHeader,
  DrawerTitle
} from '@/components/ui/drawer';
import { cn } from '@/lib/utils';
import { Skeleton } from '@/components/ui/skeleton';
import { invoke } from '@tauri-apps/api/core';
import { WeatherLocationResult } from '@/types/commands';
import { useMediaQuery } from '@/hooks/use-media-query';
import { useToast } from '@/hooks/use-toast';

// Add skeleton components at the top of the file after imports
const AccountSkeleton = () => (
  <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
    <div className="flex items-center gap-3 mb-2">
      <Skeleton className="w-6 h-6 dark:bg-slate-700" />
      <Skeleton className="h-8 w-32 dark:bg-slate-700" />
    </div>
    <Skeleton className="h-4 w-48 mb-4 dark:bg-slate-700" />
    <Skeleton className="h-10 w-full dark:bg-slate-700" />
  </div>
);

const WeatherDetailsSkeleton = () => (
  <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
    <div className="flex items-center gap-3 mb-2">
      <Skeleton className="w-6 h-6 dark:bg-slate-700" />
      <Skeleton className="h-8 w-40 dark:bg-slate-700" />
    </div>
    <Skeleton className="h-4 w-56 mb-4 dark:bg-slate-700" />
    <div className="space-y-4">
      <Skeleton className="h-10 w-full dark:bg-slate-700" />
      <Skeleton className="h-10 w-full dark:bg-slate-700" />
      <Skeleton className="h-10 w-full dark:bg-slate-700" />
      <Skeleton className="h-10 w-full dark:bg-slate-700" />
    </div>
  </div>
);

const PreferencesSkeleton = () => (
  <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
    <div className="flex items-center gap-3 mb-2">
      <Skeleton className="w-6 h-6 dark:bg-slate-700" />
      <Skeleton className="h-8 w-36 dark:bg-slate-700" />
    </div>
    <Skeleton className="h-4 w-44 mb-4 dark:bg-slate-700" />
    <div className="space-y-4">
      <div>
        <Skeleton className="h-4 w-20 mb-2 dark:bg-slate-700" />
        <Skeleton className="h-10 w-full dark:bg-slate-700" />
      </div>
      <div>
        <Skeleton className="h-4 w-36 mb-2 dark:bg-slate-700" />
        <Skeleton className="h-10 w-full dark:bg-slate-700" />
      </div>
      <Skeleton className="h-10 w-full dark:bg-slate-700" />
    </div>
  </div>
);

// Define form types
type WeatherFormValues = {
  city: string;
  state: string;
  country: string;
};

// Add this at the top of the file with other type definitions
type LanguageOption = {
  label: string;
  value: string;
};

const LANGUAGE_OPTIONS: LanguageOption[] = [
  { label: 'English', value: 'en' },
  { label: 'Spanish', value: 'es' },
  { label: 'French', value: 'fr' }
];

// Add this alongside the LANGUAGE_OPTIONS
const UNIT_OPTIONS: Array<{ label: string; value: 'imperial' | 'metric' }> = [
  { label: 'Imperial', value: 'imperial' },
  { label: 'Metric', value: 'metric' }
];

// Then update the PreferencesFormValues type
type PreferencesFormValues = {
  language: string; // This will store the language code
  units: 'imperial' | 'metric';
};

// Add this type for settings response
type UserSettingsResponse = {
  id: string;
  userId: string;
  lat: number | null;
  lon: number | null;
  city: string | null;
  state: string | null;
  country: string | null;
  units: 'imperial' | 'metric';
  language: string;
};

interface LocationSelectionProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  locations: WeatherLocationResult[];
  onLocationSelect: (location: WeatherLocationResult) => void;
}

function LocationSelection({
  open,
  onOpenChange,
  locations,
  onLocationSelect
}: LocationSelectionProps) {
  const isDesktop = useMediaQuery('(min-width: 768px)');

  const content = (
    <div className="flex flex-col gap-4">
      {locations.map((location, index) => (
        <button
          key={index}
          onClick={() => onLocationSelect(location)}
          className="rounded-lg border border-gray-100 dark:border-gray-700 p-4 flex flex-col gap-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-left transition-colors"
        >
          <div className="flex items-start gap-3">
            <div className="rounded-full bg-blue-50 dark:bg-blue-900/50 p-2">
              <MapPin className="size-5 text-blue-700 dark:text-blue-400" />
            </div>
            <div className="flex flex-col gap-1">
              <h4 className="font-medium">{location.name}</h4>
              <p className="text-gray-600 dark:text-gray-400">
                {location.state}, {location.country}
              </p>
              <p className="text-gray-500 dark:text-gray-500 text-sm">
                {location.lat.toFixed(4)}, {location.lon.toFixed(4)}
              </p>
            </div>
          </div>
        </button>
      ))}
    </div>
  );

  if (isDesktop) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-[600px] max-h-[85vh] flex flex-col">
          <DialogHeader className="border-b pb-4">
            <div className="flex items-center justify-between">
              <DialogTitle className="text-md font-normal leading-normal tracking-wide -mt-3">
                Select Location
              </DialogTitle>
              <DialogClose className="opacity-70 hover:opacity-100" />
            </div>
          </DialogHeader>
          <div className="px-1 py-4 overflow-y-auto">{content}</div>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange}>
      <DrawerContent className="max-h-[85vh] flex flex-col">
        <DrawerHeader className="border-b flex-shrink-0">
          <div className="flex items-center justify-between">
            <DrawerTitle className="text-md font-normal leading-normal tracking-wide">
              Select Location
            </DrawerTitle>
            <DrawerClose className="opacity-70 hover:opacity-100" />
          </div>
        </DrawerHeader>
        <div className="px-4 py-6 flex-1 overflow-y-auto">{content}</div>
        <div className="p-4 border-t flex-shrink-0">
          <DrawerClose asChild>
            <Button variant="outline" className="w-full">
              Close
            </Button>
          </DrawerClose>
        </div>
      </DrawerContent>
    </Drawer>
  );
}

export default function Settings() {
  const { logout, login, isAuthenticated, isLoading, getToken } =
    KindeAuth.useKindeAuth();

  const { toast } = useToast();

  // Setup forms
  const weatherForm = useForm<WeatherFormValues>({
    defaultValues: {
      city: '',
      state: '',
      country: ''
    }
  });

  // Update the preferences form initialization
  const preferencesForm = useForm<PreferencesFormValues>({
    defaultValues: {
      language: 'en',
      units: 'imperial'
    }
  });

  // Add these new state variables
  const [locations, setLocations] = useState<WeatherLocationResult[]>([]);
  const [isLocationSelectOpen, setIsLocationSelectOpen] = useState(false);
  const [selectedLocation, setSelectedLocation] =
    useState<WeatherLocationResult | null>(null);

  // Add state for existing settings
  const [existingSettings, setExistingSettings] =
    useState<UserSettingsResponse | null>(null);

  // Fetch existing settings on component mount
  useEffect(() => {
    const fetchSettings = async () => {
      if (!isAuthenticated || !getToken) return;

      try {
        const token = await getToken();
        const settings = await invoke<UserSettingsResponse>(
          'get_user_settings',
          { token }
        );

        console.log('Fetched settings:', settings);

        setExistingSettings(settings);

        // Update form values with existing settings
        if (settings) {
          console.log('Setting form values:', {
            language: settings.language,
            units: settings.units
          });

          preferencesForm.setValue('language', settings.language);
          preferencesForm.setValue('units', settings.units);

          // If we have location data, set both the form values and selectedLocation
          if (settings.city) {
            weatherForm.setValue('city', settings.city);
            weatherForm.setValue('state', settings.state || '');
            weatherForm.setValue('country', settings.country || '');

            // Set selectedLocation if we have all required location data
            if (settings.lat !== null && settings.lon !== null) {
              setSelectedLocation({
                lat: settings.lat,
                lon: settings.lon,
                name: settings.city,
                state: settings.state || '',
                country: settings.country || ''
              });
            }
          }
        }
      } catch (error) {
        console.error('Error fetching settings:', error);
        toast({
          variant: 'destructive',
          description: 'Failed to load settings'
        });
      }
    };

    fetchSettings();
  }, [isAuthenticated, getToken]);

  // Update the weather form submit handler
  const onWeatherSubmit = async (data: WeatherFormValues) => {
    if (!isAuthenticated || !getToken) return;

    try {
      const token = await getToken();
      const response = await invoke<Array<WeatherLocationResult>>(
        'get_weather_location',
        {
          token,
          location: {
            city: data.city,
            state: data.state,
            country: data.country
          }
        }
      );

      if (response.length === 1) {
        setSelectedLocation(response[0]);

        const formValues = preferencesForm.getValues(); // Get all form values at once

        const settingsData = {
          lat: response[0].lat,
          lon: response[0].lon,
          city: response[0].name,
          state: response[0].state,
          country: response[0].country,
          units: formValues.units,
          language: formValues.language
        };

        const method = existingSettings?.userId
          ? 'update_user_settings'
          : 'create_user_settings';

        await invoke(method, {
          token,
          settings: settingsData
        });

        setExistingSettings((prev) => ({
          ...prev!,
          ...settingsData
        }));

        toast({
          description: 'Location updated'
        });
      } else if (response.length > 1) {
        setLocations(response);
        setIsLocationSelectOpen(true);
      } else {
        toast({
          variant: 'destructive',
          description: 'No location found'
        });
      }
    } catch (error) {
      console.error('Error updating location:', error);
      toast({
        variant: 'destructive',
        description: 'Failed to update location'
      });
    }
  };

  // Update the preferences form submit handler
  const onPreferencesSubmit = async (data: PreferencesFormValues) => {
    if (!isAuthenticated || !getToken) return;

    try {
      const token = await getToken();
      const settingsData = {
        lat: selectedLocation?.lat || null,
        lon: selectedLocation?.lon || null,
        city: selectedLocation?.name || null,
        state: selectedLocation?.state || null,
        country: selectedLocation?.country || null,
        units: data.units,
        language: data.language
      };

      const method = existingSettings
        ? 'update_user_settings'
        : 'create_user_settings';

      await invoke(method, {
        token,
        settings: settingsData
      });

      setExistingSettings((prev) => ({
        ...prev!,
        ...settingsData
      }));

      toast({
        description: 'Preferences updated'
      });
    } catch (error) {
      console.error('Error updating preferences:', error);
      toast({
        variant: 'destructive',
        description: 'Failed to update preferences'
      });
    }
  };

  // Update the location selection handler
  const handleLocationSelect = async (location: WeatherLocationResult) => {
    if (!isAuthenticated || !getToken) return;

    try {
      setSelectedLocation(location);
      setIsLocationSelectOpen(false);

      const token = await getToken();
      const formValues = preferencesForm.getValues(); // Get all form values at once

      const settingsData = {
        lat: location.lat,
        lon: location.lon,
        city: location.name,
        state: location.state,
        country: location.country,
        units: formValues.units,
        language: formValues.language
      };

      const method = existingSettings
        ? 'update_user_settings'
        : 'create_user_settings';

      await invoke(method, {
        token,
        settings: settingsData
      });

      setExistingSettings((prev) => ({
        ...prev!,
        ...settingsData
      }));

      toast({
        description: 'Location updated'
      });
    } catch (error) {
      console.error('Error updating location:', error);
      toast({
        variant: 'destructive',
        description: 'Failed to update location'
      });
    }
  };

  return (
    <div className="space-y-8 max-w-4xl mx-auto p-6">
      {/* Account Section */}
      {isLoading ? (
        <AccountSkeleton />
      ) : (
        <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
          <div className="flex items-center gap-3 mb-2">
            <ArrowRight
              className={cn(
                'w-6 h-6',
                isAuthenticated ? 'text-red-500' : 'text-orange-500'
              )}
            />
            <h2 className="text-2xl font-bold">Account</h2>
          </div>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            Manage your account settings
          </p>
          {isAuthenticated ? (
            <Button
              variant="outline"
              className="w-full hover:border-red-500"
              onClick={() => logout()}
            >
              Logout
            </Button>
          ) : (
            <Button
              variant="outline"
              className="w-full hover:border-orange-500"
              onClick={() => login()}
            >
              Login
            </Button>
          )}
        </div>
      )}

      {/* Weather Details Section */}
      {isLoading ? (
        <WeatherDetailsSkeleton />
      ) : (
        <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
          <div className="flex items-center gap-3 mb-2">
            <Cloud className="w-6 h-6 text-blue-500" />
            <h2 className="text-2xl font-bold">Weather Details</h2>
          </div>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            Set your default weather location
          </p>

          {/* Show selected location if exists */}
          {selectedLocation && (
            <div className="mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-100 dark:border-blue-800">
              <div className="flex items-center gap-2">
                <MapPin className="w-4 h-4 text-blue-500" />
                <span className="font-medium">Selected Location:</span>
              </div>
              <div className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                {selectedLocation.name}, {selectedLocation.state},{' '}
                {selectedLocation.country}
              </div>
              <div className="mt-0.5 text-xs text-gray-500">
                {selectedLocation.lat.toFixed(4)},{' '}
                {selectedLocation.lon.toFixed(4)}
              </div>
            </div>
          )}

          <Form {...weatherForm}>
            <form
              onSubmit={weatherForm.handleSubmit(onWeatherSubmit)}
              className="space-y-4"
            >
              <FormField
                control={weatherForm.control}
                name="city"
                rules={{ required: 'City is required' }}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>City</FormLabel>
                    <FormControl>
                      <Input placeholder="Orlando" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={weatherForm.control}
                name="state"
                rules={{ required: 'State is required' }}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>State</FormLabel>
                    <FormControl>
                      <Input placeholder="Florida" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={weatherForm.control}
                name="country"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Country</FormLabel>
                    <FormControl>
                      <Input placeholder="Country code e.g., US" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button
                type="submit"
                variant="outline"
                className="w-full hover:border-blue-500"
              >
                {selectedLocation
                  ? 'Update Default Location'
                  : 'Get Weather Coordinates'}
              </Button>
            </form>
          </Form>
        </div>
      )}

      {/* Preferences Section */}
      {isLoading ? (
        <PreferencesSkeleton />
      ) : (
        <div className="bg-white dark:bg-gray-900 rounded-lg p-6 shadow">
          <div className="flex items-center gap-3 mb-2">
            <Settings2 className="w-6 h-6 text-green-500" />
            <h2 className="text-2xl font-bold">Preferences</h2>
          </div>
          <p className="text-gray-500 dark:text-gray-400 mb-4">
            Set your app preferences
          </p>
          <Form {...preferencesForm}>
            <form
              onSubmit={preferencesForm.handleSubmit(onPreferencesSubmit)}
              className="space-y-4"
            >
              <FormField
                control={preferencesForm.control}
                name="language"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Language</FormLabel>
                    <FormControl>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="outline"
                            className="w-full justify-between"
                          >
                            {LANGUAGE_OPTIONS.find(
                              (lang) => lang.value === field.value
                            )?.label || 'Select language'}
                            <ChevronDown className="h-4 w-4 opacity-50" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className="w-full min-w-[8rem]">
                          {LANGUAGE_OPTIONS.map((option) => (
                            <DropdownMenuItem
                              key={option.value}
                              onClick={() => field.onChange(option.value)}
                            >
                              {option.label}
                            </DropdownMenuItem>
                          ))}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </FormControl>
                  </FormItem>
                )}
              />
              <FormField
                control={preferencesForm.control}
                name="units"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Units of Measurement</FormLabel>
                    <FormControl>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="outline"
                            className="w-full justify-between"
                          >
                            {UNIT_OPTIONS.find(
                              (unit) => unit.value === field.value
                            )?.label || 'Select units'}
                            <ChevronDown className="h-4 w-4 opacity-50" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent className="w-full min-w-[8rem]">
                          {UNIT_OPTIONS.map((option) => (
                            <DropdownMenuItem
                              key={option.value}
                              onClick={() => field.onChange(option.value)}
                            >
                              {option.label}
                            </DropdownMenuItem>
                          ))}
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </FormControl>
                  </FormItem>
                )}
              />
              <Button
                type="submit"
                variant="outline"
                className="w-full hover:border-green-500"
              >
                Save Preferences
              </Button>
            </form>
          </Form>
        </div>
      )}

      <LocationSelection
        open={isLocationSelectOpen}
        onOpenChange={setIsLocationSelectOpen}
        locations={locations}
        onLocationSelect={handleLocationSelect}
      />
    </div>
  );
}
