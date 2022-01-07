% GRID-PRODUCT-UPDATE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-update** — Updates existing products.

SYNOPSIS
========

**grid product update** \[**FLAGS**\] \[**OPTIONS**\] <{PRODUCT_ID|**--file** FILENAME}>

DESCRIPTION
===========

Updates existing products. This command requires the `PRODUCT_ID` argument 
to specify the unique identifier for the product that is to be retrieved, 
or the `--file` option to specify the path to a YAML file or GDSN XML File
containing a list of products. if the `PRODUCT_ID` argument is specified
then properties can be specified using the available options.

ARGS
====

`PRODUCT_ID`
: Unique identifier of the product. Conflicts with `--file`.

FLAGS
=====

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output.

OPTIONS
=======

`-f`,`--file`
: Path to a YAML file containing a list of products.

`-k`, `--key`
: Base name or path to a private signing key file.

`--namespace`
: Namespace of the product (default: "GS1"). Conflicts with `--file`.

`--property`
: A product property (format: key=value). Conflicts with `--file`.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

EXAMPLES
========

Products can be updated by using the `update` command.

Using command-line arguments:
```
$ grid product update 762111177704 \
    --property width=10
    --property length=10
    --property depth=10
```

Using a YAML file:
```
$ grid product update --file products.yaml
```

Using an XML file:
```
$ grid product update --file products.xml
```

Sample YAML file describing products:
```
- product_namespace: "GS1"
  product_id: "762111177704"
  properties:
    - name: "length"
      data_type: "NUMBER"
      number_value: 8
    - name: "width"
      data_type: "NUMBER"
      number_value: 12
    - name: "height"
      data_type: "NUMBER"
      number_value: 4
- product_namespace: "GS1"
  product_id: "881334009880"
  properties:
    - name: "length"
      data_type: "NUMBER"
      number_value: 8
    - name: "width"
      data_type: "NUMBER"
      number_value: 12
    - name: "height"
      data_type: "NUMBER"
      number_value: 12
```

Sample XML file describing a list of products:
```
<gridTradeItems xmlns:ns0="urn:gs1:gdsn:food_and_beverage_ingredient:xsd:3" xmlns:ns10="urn:gs1:gdsn:trade_item_hierarchy:xsd:3" xmlns:ns11="urn:gs1:gdsn:trade_item_lifespan:xsd:3" xmlns:ns12="urn:gs1:gdsn:trade_item_measurements:xsd:3" xmlns:ns13="urn:gs1:gdsn:trade_item_temperature_information:xsd:3" xmlns:ns2="urn:gs1:gdsn:consumer_instructions:xsd:3" xmlns:ns3="urn:gs1:gdsn:food_and_beverage_preparation_serving:xsd:3" xmlns:ns4="urn:gs1:gdsn:marketing_information:xsd:3" xmlns:ns5="urn:gs1:gdsn:nutritional_information:xsd:3" xmlns:ns6="urn:gs1:gdsn:packaging_marking:xsd:3" xmlns:ns7="urn:gs1:gdsn:place_of_item_activity:xsd:3" xmlns:ns8="urn:gs1:gdsn:referenced_file_detail_information:xsd:3" xmlns:ns9="urn:gs1:gdsn:trade_item_description:xsd:3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="gridTradeItems.xsd">
    <tradeItem>
        <gtin>013600000929</gtin>
        <isTradeItemABaseUnit>true</isTradeItemABaseUnit>
        <brandOwner>
            <gln>0000000000000</gln>
            <partyName>MyOrganization</partyName>
        </brandOwner>
        <informationProviderOfTradeItem>
            <gln>0000000000005</gln>
            <partyName>OtherOrganization</partyName>
        </informationProviderOfTradeItem>
        <gdsnTradeItemClassification>
            <gpcCategoryCode>10000000</gpcCategoryCode>
        </gdsnTradeItemClassification>
        <targetMarket>
            <targetMarketCountryCode>
                NA
            </targetMarketCountryCode>
            <targetMarketSubdivisionCode>
                NA
            </targetMarketSubdivisionCode>
        </targetMarket>
        <tradeItemInformation>
            <extension>
                <foodAndBeverageIngredientModule>
                    <ingredientStatement languageCode="en">test ingredients</ingredientStatement>
                </foodAndBeverageIngredientModule>
                <consumerInstructionsModule>
                    <consumerInstructions>
                        <consumerStorageInstructions languageCode="en">test storage instructions</consumerStorageInstructions>
                    </consumerInstructions>
                </consumerInstructionsModule>
                <foodAndBeveragePreparationServingModule>
                    <preparationServing>
                        <preparationInstructions languageCode="en">test preparation instructions</preparationInstructions>
                        <preparationTypeCode>GRILL</preparationTypeCode>
                    </preparationServing>
                </foodAndBeveragePreparationServingModule>
                <marketingInformationModule>
                    <marketingInformation>
                        <tradeItemMarketingMessage languageCode="en">test trade item marketing message</tradeItemMarketingMessage>
                    </marketingInformation>
                </marketingInformationModule>
                <nutritionalInformationModule>
                    <nutritionalClaimDetail>
                        <nutritionalClaimTypeCode>CA</nutritionalClaimTypeCode>
                    </nutritionalClaimDetail>
                    <nutrientHeader>
                        <preparationStateCode>UNPREPARED</preparationStateCode>
                        <servingSize measurementUnitCode="GRM ">0</servingSize>
                        <servingSizeDescription languageCode="en">test serving size description</servingSizeDescription>
                    </nutrientHeader>
                </nutritionalInformationModule>
                <packagingMarkingModule>
                    <packagingMarking>
                        <packagingDate>
                            <tradeItemDateOnPackagingTypeCode>0</tradeItemDateOnPackagingTypeCode>
                        </packagingDate>
                    </packagingMarking>
                </packagingMarkingModule>
                <placeOfItemActivityModule>
                    <placeOfProductActivity>
                        <countryOfOrigin>
                            <countryCode>US</countryCode>
                        </countryOfOrigin>
                    </placeOfProductActivity>
                </placeOfItemActivityModule>
                <referencedFileDetailInformationModule>
                    <referencedFileHeader>
                        <referencedFileTypeCode>PRODUCT_IMAGE</referencedFileTypeCode>
                        <uniformResourceIdentifier>https://grid.hyperledger.org/assets/grid_wordmark.svg</uniformResourceIdentifier>
                    </referencedFileHeader>
                </referencedFileDetailInformationModule>
                <tradeItemDescriptionModule>
                    <tradeItemDescriptionInformation>
                        <additionalTradeItemDescription languageCode="en">test additional trade item description</additionalTradeItemDescription>
                        <descriptionShort languageCode="en">test description short</descriptionShort>
                        <functionalName languageCode="en">test functional name</functionalName>
                        <regulatedProductName languageCode="en">test regulated product name</regulatedProductName>
                        <tradeItemDescription languageCode="en">test trade item description</tradeItemDescription>
                        <brandNameInformation>
                            <brandName>Org Brand</brandName>
                        </brandNameInformation>
                    </tradeItemDescriptionInformation>
                </tradeItemDescriptionModule>
                <tradeItemHierarchyModule>
                    <tradeItemHierarchy>
                        <quantityOfInnerPack>0</quantityOfInnerPack>
                        <quantityOfTradeItemsPerPallet>0</quantityOfTradeItemsPerPallet>
                    </tradeItemHierarchy>
                </tradeItemHierarchyModule>
                <tradeItemLifespanModule>
                    <tradeItemLifespan>
                        <minimumTradeItemLifespanFromTimeOfArrival>0</minimumTradeItemLifespanFromTimeOfArrival>
                    </tradeItemLifespan>
                </tradeItemLifespanModule>
                <tradeItemMeasurementsModule>
                    <tradeItemMeasurements>
                        <depth measurementUnitCode="3">0</depth>
                        <height measurementUnitCode="3">0</height>
                        <netContent measurementUnitCode="3">0</netContent>
                        <width measurementUnitCode="3">0</width>
                        <tradeItemWeight>
                            <grossWeight measurementUnitCode="3">0</grossWeight>
                            <netWeight measurementUnitCode="3">0</netWeight>
                        </tradeItemWeight>
                    </tradeItemMeasurements>
                </tradeItemMeasurementsModule>
                <tradeItemTemperatureInformationModule>
                    <tradeItemTemperatureInformation>
                        <maximumTemperature temperatureMeasurementUnitCode="2">0</maximumTemperature>
                        <minimumTemperature temperatureMeasurementUnitCode="2">0</minimumTemperature>
                    </tradeItemTemperatureInformation>
                </tradeItemTemperatureInformationModule>
                <avpList>
                    <stringAVP attributeName="isNutrientRelevantDataProvided">true</stringAVP>
                </avpList>
            </extension>
        </tradeItemInformation>
        <tradeItemSynchronisationDates>
            <lastChangeDateTime>1970-01-01T00:00:00</lastChangeDateTime>
            <effectiveDateTime>1970-01-01T00:00:00</effectiveDateTime>
        </tradeItemSynchronisationDates>
    </tradeItem>
</gridTradeItems>
```

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions.

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

**`GRID_PRODUCT_SCHEMA_DIR`**
: Specifies the local path to the directory containing the `GridTradeItems.xsd`
  schema used to validate the product. The default value is
  "/usr/share/grid/xsd".

SEE ALSO
========
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
