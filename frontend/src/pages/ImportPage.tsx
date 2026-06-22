/**
 * Sources page - PDF upload and bank connection.
 */

import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Building2, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AppLayout } from "@/components/layout";
import { PdfUpload } from "@/components/upload";

export function ImportPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const handleUploadComplete = () => {
    // Navigate to transactions page after successful upload
    navigate("/transactions");
  };

  return (
    <AppLayout>
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Fuentes de datos</CardTitle>
          </CardHeader>
          <CardContent>
            <Tabs defaultValue="pdf" className="w-full">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="pdf">
                  <FileText className="mr-2 h-4 w-4" />
                  Subir PDF
                </TabsTrigger>
                <TabsTrigger value="bank">
                  <Building2 className="mr-2 h-4 w-4" />
                  Conectar banco
                </TabsTrigger>
              </TabsList>
              <TabsContent value="pdf" className="mt-4">
                <PdfUpload onUploadComplete={handleUploadComplete} />
              </TabsContent>
              <TabsContent value="bank" className="mt-4">
                <p className="text-gray-600">{t("connect.description")}</p>
                <Button className="mt-4" onClick={() => navigate("/connect")}>
                  <Building2 className="mr-2 h-4 w-4" />
                  {t("belvo.connectBank")}
                </Button>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>
      </div>
    </AppLayout>
  );
}
